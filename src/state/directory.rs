use std::path;

use axum::http::StatusCode;
use query::directory::Directory;
use tokio::fs;

use crate::error::Error;

use super::AppState;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct DirTree {
    dir: Directory,
    children: Vec<DirTree>,
}

impl AppState {
    pub async fn get_dir_tree(&self) -> Result<Vec<DirTree>, Error> {
        let mut res = Vec::new();
        for root_dir in self.get_directory_parentless().await? {
            let mut root = DirTree {
                dir: root_dir,
                children: Vec::new(),
            };

            let mut border = vec![&mut root];
            while let Some(tree) = border.pop() {
                tree.children = self
                    .get_directory_by_parent_id(tree.dir.id)
                    .await?
                    .into_iter()
                    .map(|dir| DirTree {
                        dir,
                        children: Vec::new(),
                    })
                    .collect();
                for dir in &mut tree.children {
                    border.push(dir);
                }
            }
            res.push(root);
        }
        Ok(res)
    }

    pub async fn get_directory_all(&self) -> Result<Vec<Directory>, Error> {
        Ok(self.pool.queryable().get_directory_all().await?)
    }
    pub async fn get_directory_by_id(&self, id: i64) -> Result<Directory, Error> {
        Ok(self.pool.queryable().get_directory_by_id(id).await?)
    }
    pub async fn get_directory_parentless(&self) -> Result<Vec<Directory>, Error> {
        Ok(self.pool.queryable().get_directory_parentless().await?)
    }
    pub async fn get_directory_by_parent_id(&self, par_id: i64) -> Result<Vec<Directory>, Error> {
        Ok(self
            .pool
            .queryable()
            .get_directory_by_parent_id(par_id)
            .await?)
    }
    pub async fn directory_ancestors(&self, dir_id: i64) -> Result<Vec<Directory>, Error> {
        Ok(self.pool.queryable().directory_ancestors(dir_id).await?)
    }

    pub async fn register_directory(
        &self,
        path: String,
        name: Option<String>,
    ) -> Result<Directory, Error> {
        let mut transaction = self.pool.transaction().await?;
        let path = path::Path::new(&path);

        let mut new_dir = Directory::new(
            None,
            name.unwrap_or(
                path.file_name()
                    .and_then(|i| i.to_str())
                    .ok_or(Error::StatusCode(
                        StatusCode::BAD_REQUEST,
                        "Invalid Path".to_string(),
                    ))?
                    .to_owned(),
            ),
            path.to_str()
                .ok_or(Error::StatusCode(
                    StatusCode::BAD_REQUEST,
                    "Invalid Path".to_string(),
                ))?
                .to_owned(),
        );
        match {
            transaction
                .queryable()
                .insert_directory(&mut new_dir)
                .await?;

            if fs::metadata(path).await.is_ok_and(|f| f.is_dir()) {
            } else {
                fs::create_dir_all(path).await?;
            }
            Ok(new_dir)
        } {
            Err(err) => {
                transaction.rollback().await?;
                Err(err)
            }
            Ok(d) => {
                transaction.commit().await?;
                self.add_missing(d.clone()).await?;
                Ok(d)
            }
        }
    }

    pub async fn unregister_directory(&self, id: i64) -> Result<(), Error>{
        Ok(self.pool.queryable().delete_directory(id).await?)
    }

    // Create a new directory inside another directory
    pub async fn create_directory(&self, par_id: i64, name: String) -> Result<Directory, Error> {
        let mut transaction = self.pool.transaction().await?;
        match {
            let par_dir = transaction.queryable().get_directory_by_id(par_id).await?;

            let mut new_dir = Directory::new(
                Some(par_id),
                name.clone(),
                path::Path::new(&par_dir.path)
                    .join(&name)
                    .to_string_lossy()
                    .into_owned(),
            );

            fs::create_dir(&new_dir.path).await?;

            transaction
                .queryable()
                .insert_directory(&mut new_dir)
                .await?;
            Ok(new_dir)
        } {
            Err(err) => {
                transaction.rollback().await?;
                Err(err)
            }
            Ok(d) => {
                transaction.commit().await?;
                Ok(d)
            }
        }
    }
}
