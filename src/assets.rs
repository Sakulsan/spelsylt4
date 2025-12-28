#![allow(dead_code)]

use std::{collections::HashMap, io};

use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AssetMap {
    assets: Vec<AssetConfig>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct LoadedAssetMap {
    assets: HashMap<String, AssetConfig>,
}

impl LoadedAssetMap {
    pub fn get_all_assets(&self) -> impl Iterator<Item = &AssetConfig> {
        self.assets.values()
    }

    pub fn save(&self) -> io::Result<()> {
        let mut f = std::fs::File::create("assets/assetconf.json")?;
        let x: AssetMap = self.clone().into();
        serde_json::to_writer(&mut f, &x)?;
        Ok(())
    }

    pub fn load() -> io::Result<Self> {
        let f = std::fs::File::open("assets/assetconf.json")?;
        let asset_map: AssetMap = serde_json::from_reader(f)?;
        Ok(asset_map.into())
    }
}

impl From<LoadedAssetMap> for AssetMap {
    fn from(val: LoadedAssetMap) -> Self {
        AssetMap {
            assets: val.assets.into_values().collect(),
        }
    }
}

impl From<AssetMap> for LoadedAssetMap {
    fn from(val: AssetMap) -> Self {
        LoadedAssetMap {
            assets: val
                .assets
                .into_iter()
                .map(|x| (x.name.clone(), x))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetConfig {
    pub name: String,
    pub path: Option<String>,
    pub info: Option<AtlasInfo>,
    #[serde(skip)]
    pub image_handle: Option<Handle<Image>>,
    #[serde(skip)]
    pub atlas_handle: Option<Handle<TextureAtlasLayout>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AtlasInfo {
    size: (u32, u32),
    columns: u32,
    rows: u32,
}

pub fn plugin(app: &mut App) {
    app.insert_resource(LoadedAssetMap::load().unwrap_or_default());
    app.add_systems(PreStartup, init_error_tex);
}

pub fn init_error_tex(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(ErrorTexture(assets.load("sprites/placeholder.png")));
}

#[derive(Resource)]
struct ErrorTexture(Handle<Image>);

#[derive(SystemParam)]
pub struct Sylt<'w> {
    error_tex: Res<'w, ErrorTexture>,
    pub asset_map: ResMut<'w, LoadedAssetMap>,
    asset_server: Res<'w, AssetServer>,
    layouts: ResMut<'w, Assets<TextureAtlasLayout>>,
    images: Res<'w, Assets<Image>>,
}

impl<'w> Sylt<'w> {
    pub fn add_sprite(&mut self, name: &str) -> &mut AssetConfig {
        self.asset_map.assets.insert(
            name.to_string(),
            AssetConfig {
                name: name.to_string(),
                info: None,
                atlas_handle: None,
                path: None,
                image_handle: None,
            },
        );
        self.asset_map.assets.get_mut(name).unwrap()
    }

    pub fn rename_asset(&mut self, old: &str, new: &str) {
        let Some(mut conf) = self.asset_map.assets.remove(old) else {
            println!("tried to rename {old}, which doesn't exist");
            return;
        };

        conf.name = new.to_string();
        self.asset_map.assets.insert(new.to_string(), conf);
    }

    pub fn get_asset(&mut self, name: &str) -> Option<&mut AssetConfig> {
        self.asset_map.assets.get_mut(name)
    }

    pub fn update_path(&mut self, name: &str, new_path: Option<String>) {
        let Some(conf) = self.get_asset(name) else {
            return;
        };
        conf.path = new_path;
        conf.image_handle = None;
    }

    pub fn update_atlas(&mut self, name: &str, new_info: Option<AtlasInfo>) {
        let Some(conf) = self.get_asset(name) else {
            return;
        };
        conf.info = new_info;
        conf.atlas_handle = None;
    }

    pub fn get_image(&mut self, name: &str) -> Handle<Image> {
        self.get_sprite(name).image
    }

    pub fn get_sprite(&mut self, name: &str) -> Sprite {
        let conf = self
            .asset_map
            .assets
            .get_mut(name)
            .expect("This asset hasn't been defined in the map yet!");
        let AssetConfig {
            name: _,
            path,
            image_handle,
            info,
            atlas_handle,
        } = conf;
        if let Some(handle) = image_handle {
            Sprite {
                image: handle.clone(),
                texture_atlas: atlas_handle.as_mut().map(|handle| TextureAtlas {
                    layout: handle.clone(),
                    index: 0,
                }),
                ..Default::default()
            }
        } else if let Some(path) = path {
            let handle = self.asset_server.load(&*path);
            if let Some(&AtlasInfo {
                size,
                columns,
                rows,
            }) = info.as_ref()
            {
                let layout_handle = self.layouts.add(TextureAtlasLayout::from_grid(
                    size.into(),
                    columns,
                    rows,
                    None,
                    None,
                ));
                *atlas_handle = Some(layout_handle.clone());
            }
            *image_handle = Some(handle.clone());

            Sprite {
                image: handle,
                texture_atlas: atlas_handle.as_mut().map(|handle| TextureAtlas {
                    layout: handle.clone(),
                    index: 0,
                }),
                ..Default::default()
            }
        } else {
            Sprite {
                image: self.error_tex.0.clone(),
                ..Default::default()
            }
        }
    }

    pub fn save(&self) -> io::Result<()> {
        self.asset_map.save()
    }
}
