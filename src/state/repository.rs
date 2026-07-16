use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter},
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};

use crate::{
    commands::FileDiff,
    state::{Index, IndexEntry, Object, Tree, TreeEntry},
    utils::normalize_path,
};

pub struct Repository {
    pub root: PathBuf,
}

#[derive(Debug)]
pub enum Head {
    Branch { name: String },
    Commit { hash: String },
}

impl Repository {
    pub fn is_valid_root(path: &Path) -> bool {
        path.join(".rgit/objects").is_dir()
            && path.join(".rgit/refs/heads").is_dir()
            && path.join(".rgit/index").is_file()
            && path.join(".rgit/HEAD").is_file()
    }

    pub fn load() -> Result<Self> {
        let mut root = env::current_dir()?;
        while !Self::is_valid_root(&root) {
            if !root.pop() {
                return Err(anyhow!("Repository not found"));
            }
        }

        Ok(Repository { root })
    }

    pub fn ignored(&self) -> Vec<PathBuf> {
        let mut ignored = Vec::new();
        if let Ok(file) = File::open(self.root.join(".rgitignore")) {
            let reader = BufReader::new(file);

            for line in reader.lines().map_while(Result::ok) {
                ignored.push(PathBuf::from(line));
            }
        }

        ignored
    }

    pub fn head(&self) -> Result<Head> {
        let content = fs::read_to_string(self.head_file_path())?;

        if let Some(head) = content.strip_prefix("ref: ") {
            let branch = PathBuf::from(head)
                .file_name()
                .ok_or(anyhow!("Current branch not found"))?
                .to_string_lossy()
                .to_string();
            Ok(Head::Branch { name: branch })
        } else if let Ok(Object::Commit(_)) = self.load_object(&content) {
            Ok(Head::Commit { hash: content })
        } else {
            Err(anyhow!("Corrupt HEAD file"))
        }
    }

    // TODO: check correctness
    pub fn head_hash(&self) -> Result<Option<String>> {
        let content = fs::read_to_string(self.head_file_path())?;

        if let Some(head) = content.strip_prefix("ref: ") {
            let path = normalize_path(&self.root.join(".rgit").join(head));
            if path.exists() {
                let hash = fs::read_to_string(path)?;
                Ok(Some(hash))
            } else if let Some(file) = path.file_name()
                && file.to_string_lossy() == "master"
            {
                Ok(None)
            } else {
                Err(anyhow!("Corrupt HEAD file"))
            }
        } else if let Ok(Object::Commit(_)) = self.load_object(&content) {
            Ok(Some(content))
        } else {
            Err(anyhow!("Corrupt HEAD file"))
        }
    }

    pub fn current_branch_path(&self) -> Result<PathBuf> {
        if let Some(head) = fs::read_to_string(self.head_file_path())?.strip_prefix("ref: ") {
            let path = normalize_path(&self.root.join(".rgit").join(head));
            Ok(path)
        } else {
            Err(anyhow!("Corrupt HEAD file"))
        }
    }

    pub fn change_head(&self, target: &Head) -> Result<()> {
        match target {
            Head::Branch { name } => {
                fs::write(self.head_file_path(), format!("ref: refs/heads/{name}"))?;
            }
            Head::Commit { hash } => {
                fs::write(self.head_file_path(), hash)?;
            }
        }

        Ok(())
    }

    pub fn branch_path(&self, name: &str) -> PathBuf {
        normalize_path(&self.heads_dir_path().join(name))
    }

    pub fn objects_dir_path(&self) -> PathBuf {
        self.root.join(".rgit/objects")
    }

    pub fn index_file_path(&self) -> PathBuf {
        self.root.join(".rgit/index")
    }

    pub fn head_file_path(&self) -> PathBuf {
        self.root.join(".rgit/HEAD")
    }

    pub fn merge_head_file_path(&self) -> PathBuf {
        self.root.join(".rgit/MERGE_HEAD")
    }

    pub fn heads_dir_path(&self) -> PathBuf {
        self.root.join(".rgit/refs/heads")
    }

    pub fn store_object(&self, object: &Object) -> Result<String> {
        let hash = object.hash()?;
        let (dir_name, file_name) = hash.split_at(2);

        let compressed = Object::compress(&object.serialize()?)?;

        let dir_path = self.objects_dir_path().join(dir_name);
        let file_path = dir_path.join(file_name);
        fs::create_dir_all(dir_path)?;

        if !file_path.exists() {
            fs::write(&file_path, compressed)?;
        }

        Ok(hash)
    }

    pub fn load_object(&self, hash: &str) -> Result<Object> {
        let (dir, file) = hash.split_at(2);
        let path = self.objects_dir_path().join(dir).join(file);

        let compressed = fs::read(path)?;

        Object::deserialize(&Object::decompress(&compressed)?)
    }

    pub fn load_blob_bytes(&self, hash: &str) -> Result<Vec<u8>> {
        if let Object::Blob(blob) = self.load_object(hash)? {
            Ok(blob.bytes)
        } else {
            Err(anyhow!("Object is not a blob: {hash}"))
        }
    }

    pub fn load_blob_text(&self, hash: &str) -> Result<String> {
        if let Ok(text) = String::from_utf8(self.load_blob_bytes(hash)?) {
            Ok(text)
        } else {
            Ok(String::from("Binary file"))
        }
    }

    pub fn load_index(&self) -> Result<Index> {
        let file = OpenOptions::new().read(true).open(self.index_file_path())?;
        let reader = BufReader::new(file);
        let index = serde_json::from_reader(reader).unwrap_or(Index::new());

        Ok(index)
    }

    pub fn store_index(&self, index: &Index) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.index_file_path())?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &index)?;

        Ok(())
    }

    pub fn load_index_from_commit(&self, hash: &str) -> Result<Index> {
        let mut index = Index::new();
        let mut stack = Vec::new();

        if let Object::Commit(commit) = self.load_object(hash)? {
            if let Object::Tree(tree) = self.load_object(&commit.tree_hash)? {
                stack.push((tree, PathBuf::new()));
            }

            while let Some((tree, path)) = stack.pop() {
                for entry in tree.entries {
                    if entry.object_type == "Blob" {
                        index.entries.insert(
                            path.join(entry.name),
                            IndexEntry {
                                hash: entry.object_hash,
                                size: 0,
                                mtime: 0,
                            },
                        );
                    } else if entry.object_type == "Tree"
                        && let Object::Tree(subtree) = self.load_object(&entry.object_hash)?
                    {
                        stack.push((subtree, path.join(entry.name)));
                    }
                }
            }

            Ok(index)
        } else {
            Err(anyhow!("Object is not a commit: {hash}"))
        }
    }

    pub fn store_index_tree(&self, index: &Index) -> Result<String> {
        let mut stack = Vec::<(String, Tree)>::new();
        stack.push((
            String::from("root"),
            Tree {
                entries: Vec::new(),
            },
        ));

        for (name, entry) in &index.entries {
            let path = PathBuf::from(name);
            let components = path
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect::<Vec<_>>();

            let file = components
                .last()
                .ok_or(anyhow!("Last element not found"))?
                .to_string();
            let mut i = 0;

            while i + 1 < stack.len() && i < components.len() && stack[i + 1].0 == components[i] {
                i += 1;
            }

            while i + 1 < stack.len() {
                if let Some(last) = stack.pop()
                    && let Some(second_to_last) = stack.last_mut()
                {
                    second_to_last.1.entries.push(TreeEntry {
                        object_type: String::from("Tree"),
                        object_hash: self.store_object(&Object::Tree(last.1))?,
                        name: last.0,
                    });
                }
            }

            stack.extend(components[i..components.len() - 1].iter().map(|c| {
                (
                    c.to_string(),
                    Tree {
                        entries: Vec::new(),
                    },
                )
            }));

            if let Some(last) = stack.last_mut() {
                last.1.entries.push(TreeEntry {
                    object_type: String::from("Blob"),
                    object_hash: entry.hash.clone(),
                    name: file,
                });
            }
        }

        while 1 < stack.len() {
            if let Some(last) = stack.pop()
                && let Some(second_to_last) = stack.last_mut()
            {
                second_to_last.1.entries.push(TreeEntry {
                    object_type: String::from("Tree"),
                    object_hash: self.store_object(&Object::Tree(last.1))?,
                    name: last.0,
                });
            }
        }

        if let Some(last) = stack.pop() {
            Ok(self.store_object(&Object::Tree(last.1))?)
        } else {
            Err(anyhow!("Error at stack"))
        }
    }

    pub fn staged_changes(&self) -> Result<FileDiff> {
        let mut diff = FileDiff {
            added: Vec::new(),
            deleted: Vec::new(),
            modified: Vec::new(),
        };

        let current_index = self.load_index()?;
        let head_index = if let Some(head_hash) = self.head_hash()? {
            self.load_index_from_commit(&head_hash)?
        } else {
            Index::new()
        };

        for (name, index_entry) in &current_index.entries {
            if let Some(head_entry) = head_index.entries.get(name) {
                if index_entry.hash != head_entry.hash {
                    diff.modified.push(name.clone());
                }
            } else {
                diff.added.push(name.clone());
            }
        }

        for (name, _) in head_index.entries {
            if !current_index.entries.contains_key(&name) {
                diff.deleted.push(name);
            }
        }

        Ok(diff)
    }
}
