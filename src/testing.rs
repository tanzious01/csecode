use crate::proxied_reqwest::create_client;
use crate::robloxian;
use crate::robloxian::FriendsList;
use crate::robloxian::IoThings;
use crate::robloxian::Robloxian;
use futures::future::join_all;
use futures::future::ok;
use indicatif;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use reqwest::Client;
use reqwest::Result;
use robloxian::IdNameHash;
use robloxian::JointJson;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::sleep;
pub async fn get_friends(
    id: &u64,
    community: Arc<Mutex<HashMap<u64, Vec<u64>>>>,
    name_map: Arc<Mutex<HashMap<u64, String>>>,
    my_client: Arc<Client>,
) -> reqwest::Result<()> {
    let url = format!("https://friends.roblox.com/v1/users/{}/friends", id);
    let mut retries = 0;
    let response_text = loop {
        match my_client.get(&url).send().await {
            Ok(response) => break response.text().await?,
            Err(e) if retries < 5 => {
                retries += 1;
                let delay = Duration::from_secs(2u64.pow(retries));
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    };
    let friends_list: FriendsList = serde_json::from_str(&response_text).unwrap();
    let friend_vec: Vec<u64> = friends_list.data.iter().map(|id| id.id).collect();
    community.lock().await.insert(*id, friend_vec);
    let name_map = Arc::clone(&name_map);
    futures::future::join_all(friends_list.data.into_iter().map(move |friend| {
        let name_map = Arc::clone(&name_map);
        async move {
            name_map.lock().await.insert(friend.id, friend.name);
        }
    }))
    .await;
    Ok(())
}

pub async fn get_fof(
    ids: Vec<u64>,
    community: &mut HashMap<u64, Vec<u64>>,
    name_map: &mut HashMap<u64, String>,
) -> reqwest::Result<()> {
    let community_arc = Arc::new(Mutex::new(HashMap::new()));
    let name_map_arc = Arc::new(Mutex::new(HashMap::new()));
    let my_client = Arc::new(create_client().await?);

    let pb = ProgressBar::new(ids.len() as u64);

    let pb_arc = Arc::new(pb.clone());
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("**-"),
    );

    let pb_arc = Arc::new(pb);

    let tasks: Vec<_> = ids
        .into_iter()
        .map(|id| {
            let community = Arc::clone(&community_arc);
            let name_map = Arc::clone(&name_map_arc);
            let my_client = Arc::clone(&my_client);
            let pb_clone = Arc::clone(&pb_arc);

            tokio::spawn(async move {
                let result = get_friends(&id, community, name_map, my_client).await;
                pb_clone.inc(1);
                pb_clone.set_message(format!("Processed ID: {}", id));
                result
            })
        })
        .collect();

    for task in tasks {
        match task.await {
            Ok(result) => match result {
                Ok(value) => {}

                Err(e) => {
                    eprintln!("Error in get_friends: {:?}", e)
                }
            },
            Err(e) => {
                eprintln!("Join Error {:?}", e)
            }
        }
    }

    community.extend(community_arc.lock().await.drain());
    name_map.extend(name_map_arc.lock().await.drain());
    pb_arc.finish_with_message("All tasks completed");
    Ok(())
}

pub async fn get_friends_interface(id: u64) -> reqwest::Result<()> {
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
            get_fof(
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
            create_client().await.unwrap(),
        )
        .await
        .unwrap();
        let ids = load_community.user_friends.get(&id).unwrap();
        get_fof(
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
}
