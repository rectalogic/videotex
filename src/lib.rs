use std::cell::RefCell;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        render_asset::RenderAssets,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        renderer::RenderQueue,
        texture::GpuImage,
    },
    window::WindowResolution,
};
use wasm_bindgen::prelude::*;
use wgpu_types::{
    CopyExternalImageDestInfo, CopyExternalImageSourceInfo, ExternalImageSource, Origin2d,
    Origin3d, PredefinedColorSpace, TextureAspect,
};

#[wasm_bindgen]
pub fn start() {
    console_error_panic_hook::set_once();
    let mut app = App::new();
    app.add_plugins((DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(800.0, 600.0),
            ..default()
        }),
        ..default()
    }),))
        .add_systems(Startup, setup);

    let render_app = app.sub_app_mut(RenderApp);
    render_app.add_systems(Render, render_videos.in_set(RenderSet::PrepareResources));

    app.run();
}

struct VideoTexture {
    video: web_sys::HtmlVideoElement,
    image_id: Option<AssetId<Image>>,
}

// wasm on web is single threaded, so this should be OK
thread_local! {
    static VIDEO_ELEMENT: RefCell<VideoTexture> = RefCell::new(VideoTexture {
        video: web_sys::window()
        .expect("window")
        .document()
        .expect("document")
        .create_element("video")
        .expect("video")
        .dyn_into::<web_sys::HtmlVideoElement>()
        .expect("web_sys::HtmlVideoElement"),
        image_id: None,
    })
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut image = Image::new_uninit(
        Extent3d {
            width: 512,
            height: 512,
            ..default()
        },
        TextureDimension::D2,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
    let image_handle = images.add(image);
    let image_id = image_handle.id();

    VIDEO_ELEMENT.with_borrow_mut(|vidtex| {
        vidtex.image_id = Some(image_id);
        vidtex.video.set_cross_origin(Some("anonymous"));
        vidtex.video.set_src("https://cdn.glitch.me/364f8e5a-f12f-4f82-a386-20e6be6b1046/bbb_sunflower_1080p_30fps_normal_10min.mp4");
        vidtex.video.set_muted(true);
        vidtex.video.set_loop(true);
        let _ = vidtex.video.play().expect("play");
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(image_handle.clone())),
    ));

    commands.spawn((PointLight::default(), Transform::from_xyz(3.0, 3.0, 2.0)));
    commands.spawn((Camera3d::default(), Transform::from_xyz(0., 0., 2.)));
}

fn render_videos(queue: Res<RenderQueue>, images: Res<RenderAssets<GpuImage>>) {
    VIDEO_ELEMENT.with_borrow(|vidtex| {
        let Some(image_id) = vidtex.image_id else {
            return;
        };
        let Some(gpu_image) = images.get(image_id) else {
            return;
        };
        queue.copy_external_image_to_texture(
            &CopyExternalImageSourceInfo {
                source: ExternalImageSource::HTMLVideoElement(vidtex.video.clone()),
                origin: Origin2d::ZERO,
                flip_y: false,
            },
            CopyExternalImageDestInfo {
                texture: &gpu_image.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
                color_space: PredefinedColorSpace::Srgb,
                premultiplied_alpha: true,
            },
            gpu_image.size,
        );
    });
}
