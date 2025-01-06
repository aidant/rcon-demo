use serde::Deserialize;

use crate::packet::{RconPacket, RconPacketType};

pub struct Minecraft {
    online_players: Vec<String>,
    whitelist_players: Vec<String>,
    whitelist_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct PlayerName {
    name: String,
}

async fn get_player_name(name: &str) -> Result<String, anyhow::Error> {
    let response = reqwest::get(format!(
        "https://api.mojang.com/users/profiles/minecraft/{}",
        urlencoding::encode(name)
    ))
    .await?
    .json::<PlayerName>()
    .await?;

    Ok(response.name)
}

impl Minecraft {
    pub fn new() -> Self {
        Minecraft {
            online_players: Vec::new(),
            whitelist_players: Vec::new(),
            whitelist_enabled: false,
        }
    }

    pub async fn handle_rcon_packet(&mut self, rcon_packet: RconPacket) -> Option<RconPacket> {
        if rcon_packet.r#type == RconPacketType::Auth {
            return Some(RconPacket {
                id: if rcon_packet.body == "no" {
                    -1
                } else {
                    rcon_packet.id
                },
                r#type: RconPacketType::AuthResponse,
                body: "".to_string(),
            });
        };

        if rcon_packet.r#type == RconPacketType::ExecCommand {
            if rcon_packet.body == "list" {
                return Some(RconPacket {
                    id: rcon_packet.id,
                    r#type: RconPacketType::ResponseValue,
                    body: format!(
                        "There are {} of a max of 20 players online: {}",
                        self.online_players.len(),
                        self.online_players.join(", ")
                    )
                    .to_string(),
                });
            }

            if rcon_packet.body == "whitelist on" {
                if self.whitelist_enabled {
                    return Some(RconPacket {
                        id: rcon_packet.id,
                        r#type: RconPacketType::ResponseValue,
                        body: format!("Whitelist is already turned on").to_string(),
                    });
                } else {
                    self.whitelist_enabled = true;
                    return Some(RconPacket {
                        id: rcon_packet.id,
                        r#type: RconPacketType::ResponseValue,
                        body: format!("Whitelist is now turned on").to_string(),
                    });
                }
            }

            if rcon_packet.body == "whitelist off" {
                if self.whitelist_enabled {
                    self.whitelist_enabled = false;
                    return Some(RconPacket {
                        id: rcon_packet.id,
                        r#type: RconPacketType::ResponseValue,
                        body: format!("Whitelist is now turned off").to_string(),
                    });
                } else {
                    return Some(RconPacket {
                        id: rcon_packet.id,
                        r#type: RconPacketType::ResponseValue,
                        body: format!("Whitelist is already turned off").to_string(),
                    });
                }
            }

            if rcon_packet.body == "whitelist list" {
                if self.whitelist_players.is_empty() {
                    return Some(RconPacket {
                        id: rcon_packet.id,
                        r#type: RconPacketType::ResponseValue,
                        body: format!("There are no whitelisted players").to_string(),
                    });
                } else {
                    return Some(RconPacket {
                        id: rcon_packet.id,
                        r#type: RconPacketType::ResponseValue,
                        body: format!(
                            "There are {} whitelisted player(s): {}",
                            self.whitelist_players.len(),
                            self.whitelist_players.join(", ")
                        )
                        .to_string(),
                    });
                }
            }

            if rcon_packet.body.starts_with("whitelist add ") {
                let player_name_raw = &rcon_packet.body[14..];

                if let Some(player_name) = get_player_name(player_name_raw).await.ok() {
                    if self.whitelist_players.contains(&player_name) {
                        return Some(RconPacket {
                            id: rcon_packet.id,
                            r#type: RconPacketType::ResponseValue,
                            body: format!("Player is already whitelisted").to_string(),
                        });
                    } else {
                        self.online_players.push(player_name.to_string());
                        self.whitelist_players.push(player_name.to_string());
                        return Some(RconPacket {
                            id: rcon_packet.id,
                            r#type: RconPacketType::ResponseValue,
                            body: format!("Added {} to the whitelist", player_name).to_string(),
                        });
                    }
                } else {
                    return Some(RconPacket {
                        id: rcon_packet.id,
                        r#type: RconPacketType::ResponseValue,
                        body: format!("That player does not exist",).to_string(),
                    });
                }
            }

            if rcon_packet.body.starts_with("whitelist remove ") {
                let player_name_raw = &rcon_packet.body[17..];

                if let Some(player_name) = get_player_name(player_name_raw).await.ok() {
                    if self.whitelist_players.contains(&player_name) {
                        self.online_players.retain(|x| *x != player_name);
                        self.whitelist_players.retain(|x| *x != player_name);
                        return Some(RconPacket {
                            id: rcon_packet.id,
                            r#type: RconPacketType::ResponseValue,
                            body: format!("Removed {} from the whitelist", player_name).to_string(),
                        });
                    } else {
                        return Some(RconPacket {
                            id: rcon_packet.id,
                            r#type: RconPacketType::ResponseValue,
                            body: format!("Player is not whitelisted").to_string(),
                        });
                    }
                } else {
                    return Some(RconPacket {
                        id: rcon_packet.id,
                        r#type: RconPacketType::ResponseValue,
                        body: format!("That player does not exist",).to_string(),
                    });
                }
            }

            if rcon_packet.body == "whitelist reload" {
                return Some(RconPacket {
                    id: rcon_packet.id,
                    r#type: RconPacketType::ResponseValue,
                    body: format!("").to_string(),
                });
            }

            return Some(RconPacket {
                id: rcon_packet.id,
                r#type: RconPacketType::ResponseValue,
                body: format!(
                    "Unknown or incomplete command, see below for error{}<--[HERE]",
                    rcon_packet.body
                )
                .to_string(),
            });
        }

        None
    }
}
