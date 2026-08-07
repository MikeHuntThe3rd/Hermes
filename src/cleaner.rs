
use crate::types::{AppState, OBJ_PTH_STR, Object};

use std::{ffi::OsString, time::Duration};

use tokio::fs as tfs;

pub async fn cleaner_subprocess(state: AppState) {
    let inf = state.db_interface;
    loop {
        tokio::time::sleep(Duration::from_mins(5)).await;

        let tracked_objs: Vec<Object> = inf.select::<i32, Object>(None)
        .await.unwrap_or(vec![]);

        let mut all_objs = if let Ok(read_dir) = tfs::read_dir(OBJ_PTH_STR).await {
            read_dir
        } else {
            continue;
        };

        while let Ok(Some(curr_obj)) = all_objs.next_entry().await {
            let dir_name = curr_obj.file_name();
            let path = curr_obj.path();
            if matches!(
                curr_obj.metadata().await, 
                Ok(mta) if !mta.is_dir()) 
            {
                println!("skipping following dir due to metadata read error: {}", path.to_string_lossy());
                continue;
            }

            if !hash_in_db(dir_name, &tracked_objs).await {
                match tfs::remove_dir_all(&path).await {
                  Ok(_) => { println!("removed unused dir at: {}", path.to_string_lossy()); },  
                  Err(e) => { 
                    println!("failed to delete dir at: {}", path.to_string_lossy());
                    println!("due to: {}", e.to_string());
                },  
                };
            }
        }
    }
}

async fn hash_in_db(dir_name: OsString, tracked_objs: &Vec<Object>) -> bool {
    for tracked_obj in tracked_objs {
        if *tracked_obj.hash == *dir_name {
            return true;
        }
    }
    return false;
}