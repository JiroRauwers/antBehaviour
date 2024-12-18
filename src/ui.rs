use bevy::{
    color::palettes::css::{AQUA, LIME, RED},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

use crate::ant::AntSettings;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
            .insert_resource(UiControls {
                show_ant_views: true,
                show_grid: false,
                show_pheromones: false,
            })
            .add_systems(Startup, setup)
            .add_systems(Update, (counter_system, button_system));
    }
}

#[derive(Component)]
struct StatsText;

#[derive(Resource)]
pub struct UiControls {
    pub show_grid: bool,
    pub show_ant_views: bool,
    pub show_pheromones: bool,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands) {
    let font = TextFont {
        font_size: 14.0,
        ..Default::default()
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.75)),
            GlobalZIndex(i32::MAX),
        ))
        .with_children(|p| {
            p.spawn((Text::default(), StatsText)).with_children(|p| {
                p.spawn((
                    TextSpan::new("Ants Count: "),
                    font.clone(),
                    TextColor(LIME.into()),
                ));
                p.spawn((TextSpan::new(""), font.clone(), TextColor(AQUA.into())));
                p.spawn((
                    TextSpan::new("\nFPS (raw): "),
                    font.clone(),
                    TextColor(LIME.into()),
                ));
                p.spawn((TextSpan::new(""), font.clone(), TextColor(AQUA.into())));
                p.spawn((
                    TextSpan::new("\nFPS (SMA): "),
                    font.clone(),
                    TextColor(LIME.into()),
                ));
                p.spawn((TextSpan::new(""), font.clone(), TextColor(AQUA.into())));
                p.spawn((
                    TextSpan::new("\nFPS (EMA): "),
                    font.clone(),
                    TextColor(LIME.into()),
                ));
                p.spawn((TextSpan::new(""), font.clone(), TextColor(AQUA.into())));
            });
            p.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            border: UiRect::all(Val::Px(5.0)),
                            // horizontally center child text
                            justify_content: JustifyContent::Center,
                            // vertically center child text
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BorderRadius::MAX,
                        BackgroundColor(NORMAL_BUTTON),
                    ))
                    .with_child((
                        Text::new("Button"),
                        font.clone(),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
            });
        });
}

fn counter_system(
    diagnostics: Res<DiagnosticsStore>,
    ants_settings: Res<AntSettings>,
    query: Single<Entity, With<StatsText>>,
    mut writer: TextUiWriter,
) {
    let text = *query;

    if ants_settings.is_changed() {
        *writer.text(text, 2) = ants_settings.n_ants.to_string();
    }

    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(raw) = fps.value() {
            *writer.text(text, 4) = format!("{raw:.2}");
        }
        if let Some(sma) = fps.average() {
            *writer.text(text, 6) = format!("{sma:.2}");
        }
        if let Some(ema) = fps.smoothed() {
            *writer.text(text, 8) = format!("{ema:.2}");
        }
    };
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                **text = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
            }
            Interaction::Hovered => {
                **text = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                **text = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}
