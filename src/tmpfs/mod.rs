mod builder;
mod inode;
mod link;
mod open;

pub use builder::Builder;

#[cfg(test)]
mod test {
    use std::io::IoSliceMut;

    use super::builder::Builder;
    use crate::Ledger;
    use wasi_common::file::{FdFlags, FileType, OFlags};
    use wasi_common::Error;

    #[tokio::test]
    async fn test() -> Result<(), Error> {
        let treefs = Builder::from(Ledger::new())
            .add("foo", None)
            .await?
            .add("foo/bar", b"abc".to_vec())
            .await?
            .add("foo/baz", b"abc".to_vec())
            .await?
            .add("foo/bat", None)
            .await?
            .add("foo/bat/qux", b"abc".to_vec())
            .await?
            .add("ack", None)
            .await?
            .add("ack/act", b"abc".to_vec())
            .await?
            .add("zip", b"abc".to_vec())
            .await?
            .build();

        let top: Vec<Result<_, _>> = treefs.readdir(0.into()).await.unwrap().collect();

        assert_eq!(top[0].as_ref().unwrap().name, ".");
        assert_eq!(top[0].as_ref().unwrap().filetype, FileType::Directory);
        assert_eq!(top[0].as_ref().unwrap().inode, 0);

        assert_eq!(top[1].as_ref().unwrap().name, "ack");
        assert_eq!(top[1].as_ref().unwrap().filetype, FileType::Directory);
        assert_eq!(top[1].as_ref().unwrap().inode, 6);

        assert_eq!(top[2].as_ref().unwrap().name, "foo");
        assert_eq!(top[2].as_ref().unwrap().filetype, FileType::Directory);
        assert_eq!(top[2].as_ref().unwrap().inode, 1);

        assert_eq!(top[3].as_ref().unwrap().name, "zip");
        assert_eq!(top[3].as_ref().unwrap().filetype, FileType::RegularFile);
        assert_eq!(top[3].as_ref().unwrap().inode, 8);

        let foo = treefs.open_dir(false, "foo").await.unwrap();
        let foo: Vec<Result<_, _>> = foo.readdir(0.into()).await.unwrap().collect();

        assert_eq!(foo[0].as_ref().unwrap().name, ".");
        assert_eq!(foo[0].as_ref().unwrap().filetype, FileType::Directory);
        assert_eq!(foo[0].as_ref().unwrap().inode, 1);

        assert_eq!(foo[1].as_ref().unwrap().name, "..");
        assert_eq!(foo[1].as_ref().unwrap().filetype, FileType::Directory);
        assert_eq!(foo[1].as_ref().unwrap().inode, 0);

        assert_eq!(foo[2].as_ref().unwrap().name, "bar");
        assert_eq!(foo[2].as_ref().unwrap().filetype, FileType::RegularFile);
        assert_eq!(foo[2].as_ref().unwrap().inode, 2);

        assert_eq!(foo[3].as_ref().unwrap().name, "bat");
        assert_eq!(foo[3].as_ref().unwrap().filetype, FileType::Directory);
        assert_eq!(foo[3].as_ref().unwrap().inode, 4);

        assert_eq!(foo[4].as_ref().unwrap().name, "baz");
        assert_eq!(foo[4].as_ref().unwrap().filetype, FileType::RegularFile);
        assert_eq!(foo[4].as_ref().unwrap().inode, 3);

        let mut qux = treefs
            .open_file(
                false,
                "foo/bat/qux",
                OFlags::empty(),
                true,
                false,
                FdFlags::empty(),
            )
            .await
            .unwrap();

        let mut buf = [0u8; 3];
        let mut bufs = [IoSliceMut::new(&mut buf)];
        let len = qux.read_vectored(&mut bufs).await.unwrap();
        assert_eq!(len, 3);
        assert_eq!(&buf, b"abc");

        Ok(())
    }
}
