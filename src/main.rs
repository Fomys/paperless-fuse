use clap::Parser;
use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::time;
use std::time::Duration;

use paperless::{correspondent, document, document_type, saved_view, tag, Paperless};
use users::{get_current_gid, get_current_uid};
#[derive(Parser)]
struct Cli {
    /// Root url of the instance
    root: String,
    /// Token to use during connection
    token: String,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let paperless = Paperless::new(&cli.root, &cli.token);

    fuser::mount2(
        HelloFS::new(paperless),
        "mount",
        &[
            MountOption::RO,
            MountOption::FSName("paperless".to_string()),
        ],
    )
    .unwrap();
}

// ino:
// 0x0xxx_xxxx_xxxx_xxxx => Special
// 0x1xxx_xxxx_xxxx_xxxx => Document
// 0x2xxx_xxxx_xxxx_xxxx => Tag
// 0x3xxx_xxxx_xxxx_xxxx => Document type
// 0x4xxx_xxxx_xxxx_xxxx => Correspondent
struct HelloFS {
    paperless: Paperless,
    inode_tags: HashMap<String, u64>,
    inode_documents: HashMap<String, u64>,
    inode_document_types: HashMap<String, u64>,
    inode_correspondents: HashMap<String, u64>,
    inode_views: HashMap<String, u64>,
}

impl HelloFS {
    pub fn new(paperless: Paperless) -> Self {
        Self {
            paperless,
            inode_tags: Default::default(),
            inode_documents: Default::default(),
            inode_document_types: Default::default(),
            inode_correspondents: Default::default(),
            inode_views: Default::default(),
        }
    }
}

fn sanitize(s: &String) -> String {
    s.replace("/", "_")
}

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &fuser::Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let mut file_attr = FileAttr {
            ino: 0,
            size: 0,
            blocks: 0,
            atime: time::UNIX_EPOCH,
            mtime: time::UNIX_EPOCH,
            ctime: time::UNIX_EPOCH,
            crtime: time::UNIX_EPOCH,
            kind: FileType::RegularFile,
            perm: 0,
            nlink: 0,
            uid: get_current_uid(),
            gid: get_current_gid(),
            rdev: 0,
            blksize: 0,
            flags: 0,
        };

        if parent == 1 {
            if name.to_str() == Some("Tags") {
                file_attr.ino = 2;
                file_attr.nlink = 2;
                file_attr.perm = 0o550;
                file_attr.kind = FileType::Directory;
            } else if name.to_str() == Some("Document type") {
                file_attr.ino = 3;
                file_attr.nlink = 2;
                file_attr.perm = 0o550;
                file_attr.kind = FileType::Directory;
            } else if name.to_str() == Some("Correspondents") {
                file_attr.ino = 4;
                file_attr.nlink = 2;
                file_attr.perm = 0o550;
                file_attr.kind = FileType::Directory;
            } else if name.to_str() == Some("Views") {
                file_attr.ino = 5;
                file_attr.nlink = 2;
                file_attr.perm = 0o550;
                file_attr.kind = FileType::Directory;
            }
        } else if parent == 2 {
            if let Some(ino) = self.inode_tags.get(&name.to_str().unwrap().to_string()) {
                file_attr.nlink = 2;
                file_attr.ino = *ino;
                file_attr.kind = FileType::Directory;
                file_attr.perm = 0o550;
            }
        } else if parent == 3 {
            if let Some(ino) = self
                .inode_document_types
                .get(&name.to_str().unwrap().to_string())
            {
                file_attr.nlink = 2;
                file_attr.ino = *ino;
                file_attr.kind = FileType::Directory;
                file_attr.perm = 0o550;
            }
        } else if parent == 4 {
            if let Some(ino) = self
                .inode_correspondents
                .get(&name.to_str().unwrap().to_string())
            {
                file_attr.nlink = 2;
                file_attr.ino = *ino;
                file_attr.kind = FileType::Directory;
                file_attr.perm = 0o550;
            }
        } else if parent == 5 {
            if let Some(ino) = self.inode_views.get(&name.to_str().unwrap().to_string()) {
                file_attr.nlink = 2;
                file_attr.ino = *ino;
                file_attr.kind = FileType::Directory;
                file_attr.perm = 0o550;
            }
        } else if (parent & 0xF000_0000_0000_0000 == 0x2000_0000_0000_0000)
            | (parent & 0xF000_0000_0000_0000 == 0x3000_0000_0000_0000)
            | (parent & 0xF000_0000_0000_0000 == 0x4000_0000_0000_0000)
            | (parent & 0xF000_0000_0000_0000 == 0x5000_0000_0000_0000)
        {
            if let Some(ino) = self
                .inode_documents
                .get(&name.to_str().unwrap().to_string())
            {
                file_attr.size = self
                    .paperless
                    .document_size(document::Id::from(ino & 0x0FFF_FFFF_FFFF_FFFF))
                    as u64;
                file_attr.nlink = 1;
                file_attr.ino = *ino;
                file_attr.perm = 0o440;
                file_attr.kind = FileType::RegularFile;
                file_attr.blksize = 512;
            }
        };

        reply.entry(&Duration::from_secs(60), &file_attr, 0);
    }

    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: ReplyAttr) {
        let mut file_attr = FileAttr {
            ino,
            size: 0,
            blocks: 0,
            atime: time::UNIX_EPOCH,
            mtime: time::UNIX_EPOCH,
            ctime: time::UNIX_EPOCH,
            crtime: time::UNIX_EPOCH,
            kind: FileType::RegularFile,
            perm: 0,
            nlink: 0,
            uid: get_current_uid(),
            gid: get_current_gid(),
            rdev: 0,
            blksize: 0,
            flags: 0,
        };
        if (ino == 1) | (ino == 2) | (ino == 3) | (ino == 4) | (ino == 5) {
            file_attr.nlink = 2;
            file_attr.perm = 0o550;
            file_attr.kind = FileType::Directory;
        } else if ino & 0xF000_0000_0000_0000 == 0x1000_0000_0000_0000 {
            file_attr.perm = 0o550;
            file_attr.nlink = 1;
            file_attr.kind = FileType::RegularFile;
            file_attr.size = self
                .paperless
                .document_size(document::Id::from(ino & 0x0FFF_FFFF_FFFF_FFFF))
                as u64;
        } else if (ino & 0xF000_0000_0000_0000 == 0x2000_0000_0000_0000)
            | (ino & 0xF000_0000_0000_0000 == 0x3000_0000_0000_0000)
            | (ino & 0xF000_0000_0000_0000 == 0x4000_0000_0000_0000)
            | (ino & 0xF000_0000_0000_0000 == 0x5000_0000_0000_0000)
        {
            file_attr.perm = 0o550;
            file_attr.nlink = 2;
            file_attr.kind = FileType::Directory;
        }
        reply.attr(&Duration::from_secs(60), &file_attr);
    }

    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        if ino & 0xF000_0000_0000_0000 == 0x1000_0000_0000_0000 {
            let f = self
                .paperless
                .document_download(document::Id::from(ino & 0x0FFF_FFFF_FFFF_FFFF));
            reply.data(&f[offset as usize..f.len().min((offset + size as i64) as usize)]);
        }
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let mut entries = vec![
            (1, FileType::Directory, ".".to_string()),
            (1, FileType::Directory, "..".to_string()),
        ];

        if ino == 1 {
            entries.push((2, FileType::Directory, "Tags".to_string()));
            entries.push((3, FileType::Directory, "Document type".to_string()));
            entries.push((4, FileType::Directory, "Correspondents".to_string()));
            entries.push((5, FileType::Directory, "Views".to_string()));
        } else if ino == 2 {
            for tag in self.paperless.tags(tag::Filter::default()) {
                let tag = tag.unwrap();
                entries.push((
                    0x2000_0000_0000_0000 | u64::from(tag.id),
                    FileType::Directory,
                    sanitize(&tag.name),
                ));
                self.inode_tags.insert(
                    sanitize(&tag.name),
                    0x2000_0000_0000_0000 | u64::from(tag.id),
                );
            }
        } else if ino == 3 {
            for document_type in self
                .paperless
                .document_types(document_type::Filter::default())
            {
                let document_type = document_type.unwrap();
                entries.push((
                    0x3000_0000_0000_0000 | u64::from(document_type.id),
                    FileType::Directory,
                    sanitize(&document_type.name),
                ));
                self.inode_document_types.insert(
                    sanitize(&document_type.name),
                    0x3000_0000_0000_0000 | u64::from(document_type.id),
                );
            }
        } else if ino == 4 {
            for correspondent in self
                .paperless
                .correspondents(correspondent::Filter::default())
            {
                let correspondent = correspondent.unwrap();
                entries.push((
                    0x4000_0000_0000_0000 | u64::from(correspondent.id),
                    FileType::Directory,
                    sanitize(&correspondent.name),
                ));
                self.inode_correspondents.insert(
                    sanitize(&correspondent.name),
                    0x4000_0000_0000_0000 | u64::from(correspondent.id),
                );
            }
        } else if ino == 5 {
            for view in self.paperless.saved_views() {
                let view = view.unwrap();
                entries.push((
                    0x5000_0000_0000_0000 | u64::from(view.id),
                    FileType::Directory,
                    sanitize(&view.name),
                ));
                self.inode_views.insert(
                    sanitize(&view.name),
                    0x5000_0000_0000_0000 | u64::from(view.id),
                );
            }
        } else {
            let mut filter = document::Filter::default();
            if ino & 0xF000_0000_0000_0000 == 0x2000_0000_0000_0000 {
                filter.tag_id = Some(tag::Id::from(ino & 0x0FFF_FFFF_FFFF_FFFF));
            } else if ino & 0xF000_0000_0000_0000 == 0x3000_0000_0000_0000 {
                filter.document_type_id =
                    Some(document_type::Id::from(ino & 0x0FFF_FFFF_FFFF_FFFF));
            } else if ino & 0xF000_0000_0000_0000 == 0x4000_0000_0000_0000 {
                filter.correspondent_id =
                    Some(correspondent::Id::from(ino & 0x0FFF_FFFF_FFFF_FFFF));
            }
            if ino & 0xF000_0000_0000_0000 == 0x5000_0000_0000_0000 {
                filter = document::Filter::from_filter_rules(
                    &self
                        .paperless
                        .saved_view(saved_view::Id::from(ino & 0x0FFF_FFFF_FFFF_FFFF))
                        .unwrap()
                        .filter_rules,
                );
            }
            for document in self.paperless.documents(filter) {
                let document = document.unwrap();
                entries.push((
                    0x1000_0000_0000_0000 | u64::from(document.id),
                    FileType::RegularFile,
                    format!(
                        "{} - {}.pdf",
                        document.id.to_string(),
                        sanitize(&document.title)
                    ),
                ));
                self.inode_documents.insert(
                    format!(
                        "{} - {}.pdf",
                        document.id.to_string(),
                        sanitize(&document.title)
                    ),
                    0x1000_0000_0000_0000 | u64::from(document.id),
                );
            }
        }

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }
        reply.ok();
    }
}
