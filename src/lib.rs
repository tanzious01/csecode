use pyo3::prelude::*;
use std::{
    collections::HashMap,
    io::{self, Write},
};

mod authenticate;
mod proxied_reqwest;
mod robloxian;
mod testing;

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

#[pymodule]
fn rblx_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_friends, m)?)?;
    Ok(())
}
