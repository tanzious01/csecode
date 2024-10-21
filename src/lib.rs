use pyo3::prelude::*;
use std::{
    collections::HashMap,
    io::{self, Write},
    u64,
};

mod authenticate;
mod proxied_reqwest;
mod robloxian;
mod testing;
use pyo3::types::PyList;
use rayon::prelude::*;
use robloxian::{FriendsListJson, IdNameHash, IoThings, JointJson, Robloxian};
#[pyfunction]
fn get_friends(id: u64) -> PyResult<()> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let mut load_community = robloxian::FriendsListJson::load().unwrap();
        let mut name_map = IdNameHash::load().unwrap();

        if let Some(current_id) = load_community.user_friends.clone().get_mut(&id) {
            current_id.retain(|key| !load_community.user_friends.contains_key(key));
            if current_id.len() == 0 {
                println!("Robloxian already in the system");
            } else {
                println!(
                    "Found {:?}, grabbing their friends' friends",
                    name_map.names.get_mut(&id)
                );
                testing::get_fof(
                    current_id.to_owned(),
                    &mut load_community.user_friends,
                    &mut name_map.names,
                )
                .await
                .unwrap();
                let json_joint = JointJson::new(name_map.names, load_community.user_friends);
                json_joint.await.write().unwrap();
            }
        } else {
            println!("User not found, will grab their info, and their friends' info");
            robloxian::Robloxian::create_user(id, &mut name_map).await;
            robloxian::Robloxian::get_friends(
                &id,
                &mut load_community.user_friends,
                &mut name_map.names,
                proxied_reqwest::create_client().await.unwrap(),
            )
            .await
            .unwrap();
            let ids = load_community.user_friends.get(&id).unwrap();
            testing::get_fof(
                ids.to_owned(),
                &mut load_community.user_friends,
                &mut name_map.names,
            )
            .await
            .unwrap();
            let json_joint = JointJson::new(name_map.names, load_community.user_friends);
            json_joint.await.write().unwrap();
        }
        Ok(())
    })
}

#[pyfunction]
fn get_friends_vec(ids: Vec<u64>) -> PyResult<()> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let tasks: Vec<_> = ids
            .into_iter()
            .map(|id| {
                tokio::spawn(async move {
                    testing::get_friends_interface(id.to_owned()).await.unwrap();
                })
            })
            .collect();
        for task in tasks {
            task.await;
        }
        Ok(())
    })
}

#[pyfunction]
pub fn full_community() -> PyResult<Vec<(u64, u64)>> {
    let hashmap = robloxian::FriendsListJson::load().unwrap().user_friends;
    let my_vec: Vec<(u64, u64)> = hashmap
        .par_iter()
        .flat_map(|(source, target)| target.par_iter().map(move |targ| (*source, *targ)))
        .collect();
    Ok(my_vec)
}

#[pyfunction]
pub fn ego_community(id: u64) -> PyResult<Vec<(u64, u64)>> {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let hashmap = robloxian::FriendsListJson::load().unwrap().user_friends;

        if let Some(user) = hashmap.get(&id) {
            let my_vec: Vec<(u64, u64)> = user.par_iter().map(|&target| (id, target)).collect();
            return Ok(my_vec);
        } else {
            println!("User with {} not found", id);
            return Ok(Vec::new());
        };
    })
}

#[pymodule]
fn rblx_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_friends, m)?)?;
    m.add_function(wrap_pyfunction!(get_friends_vec, m)?)?;
    m.add_function(wrap_pyfunction!(full_community, m)?);
    m.add_function(wrap_pyfunction!(ego_community, m)?);
    Ok(())
}
