use iced::overlay::menu;
use iced::theme::{Checkbox, Container, Menu, PickList, Radio, Scrollable, Slider, Svg, TextInput};
use iced::widget::slider::{Handle, HandleShape};
use iced::widget::{
    button, checkbox, container, pick_list, radio, scrollable, slider, svg, text, text_input,
};
use iced::{application, Color};

macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        iced::Color::from_rgb($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0)
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Blue,
}

impl Theme {
    pub fn palette(&self) -> OCPPalette {
        match self {
            Self::Blue => OCPPalette::BLUE,
        }
    }
    pub const ALL: [Theme; 1] = [Theme::Blue];
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Theme::Blue => "Blue",
            }
        )
    }
}

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: self.palette().container_bg,
            text_color: Color::BLACK,
        }
    }
}

impl text::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: Self::Style) -> text::Appearance {
        text::Appearance { color: None }
    }
}

impl button::StyleSheet for Theme {
    type Style = ButtonStyle;

    fn active(&self, style: &ButtonStyle) -> button::Appearance {
        let palette = self.palette();

        match style {
            ButtonStyle::LightSquare => button::Appearance {
                background: Some(iced::Background::Color(palette.light_square)),
                ..Default::default()
            },
            ButtonStyle::DarkSquare => button::Appearance {
                background: Some(iced::Background::Color(palette.dark_square)),
                ..Default::default()
            },
            ButtonStyle::SelectedLightSquare => button::Appearance {
                background: Some(iced::Background::Color(palette.selected_light_square)),
                ..Default::default()
            },
            ButtonStyle::SelectedDarkSquare => button::Appearance {
                background: Some(iced::Background::Color(palette.selected_dark_square)),
                ..Default::default()
            },
            ButtonStyle::Normal => button::Appearance {
                border_width: 2.,
                border_color: palette.dark_square,
                background: Some(iced::Background::Color(palette.light_square)),
                text_color: palette.tab_label,
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        match style {
            ButtonStyle::Normal => button::Appearance {
                border_width: 2.,
                border_color: self.palette().dark_square,
                background: Some(iced::Background::Color(self.palette().dark_square)),
                text_color: self.palette().label_selected,
                ..Default::default()
            },
            _ => self.active(style),
        }
    }
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(self.palette().simple_text),
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            border_radius: 2.0.into(),
            border_width: 0.0,
            border_color: Color::WHITE,
        }
    }
}

impl text_input::StyleSheet for Theme {
    type Style = TextInput;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: iced::Background::Color(self.palette().light_square),
            border_radius: 1.0.into(),
            border_width: 1.,
            border_color: self.palette().dark_square,
            icon_color: self.palette().simple_text,
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: iced::Background::Color(self.palette().light_square),
            border_radius: 1.0.into(),
            border_width: 1.,
            border_color: self.palette().dark_square,
            icon_color: self.palette().simple_text,
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        self.palette().tab_label
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        self.palette().tab_label
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::WHITE
    }

    fn disabled(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: iced::Background::Color(self.palette().light_square),
            border_radius: 1.0.into(),
            border_width: 1.,
            border_color: self.palette().dark_square,
            icon_color: self.palette().simple_text,
        }
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        self.palette().dark_square
    }
}

impl svg::StyleSheet for Theme {
    type Style = Svg;

    fn appearance(&self, _style: &Self::Style) -> svg::Appearance {
        svg::Appearance { color: None }
    }
}

impl scrollable::StyleSheet for Theme {
    type Style = Scrollable;

    fn active(&self, _style: &Self::Style) -> scrollable::Scrollbar {
        scrollable::Scrollbar {
            background: Some(iced::Background::Color(self.palette().light_square)),
            border_radius: 1.0.into(),
            border_width: 1.,
            border_color: self.palette().light_square,
            scroller: scrollable::Scroller {
                color: self.palette().dark_square,
                border_radius: 0.0.into(),
                border_width: 1.,
                border_color: self.palette().light_square,
            },
        }
    }

    fn hovered(
        &self,
        _style: &Self::Style,
        _is_mouse_over_scrollbar: bool,
    ) -> scrollable::Scrollbar {
        scrollable::Scrollbar {
            background: Some(iced::Background::Color(self.palette().light_square)),
            border_radius: 1.0.into(),
            border_width: 1.,
            border_color: self.palette().dark_square,
            scroller: scrollable::Scroller {
                color: self.palette().dark_square,
                border_radius: 1.0.into(),
                border_width: 1.,
                border_color: Color::BLACK,
            },
        }
    }
}

impl checkbox::StyleSheet for Theme {
    type Style = Checkbox;

    fn active(&self, _style: &Self::Style, _is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: iced::Background::Color(self.palette().light_square),
            border_radius: 1.0.into(),
            border_width: 1.,
            border_color: Color::BLACK,
            icon_color: self.palette().tab_label,
            text_color: Some(self.palette().tab_label),
        }
    }

    fn hovered(&self, _style: &Self::Style, _is_checked: bool) -> checkbox::Appearance {
        checkbox::Appearance {
            background: iced::Background::Color(self.palette().dark_square),
            border_radius: 1.0.into(),
            border_width: 1.,
            border_color: Color::BLACK,
            icon_color: self.palette().label_selected,
            text_color: Some(self.palette().label_selected),
        }
    }
}

impl pick_list::StyleSheet for Theme {
    type Style = PickList;

    fn active(&self, _style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: self.palette().tab_label,
            placeholder_color: self.palette().tab_label,
            background: iced::Background::Color(self.palette().light_square),
            border_radius: 0.5.into(),
            border_width: 1.,
            border_color: self.palette().dark_square,
            handle_color: self.palette().dark_square,
        }
    }

    fn hovered(&self, _style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        pick_list::Appearance {
            text_color: self.palette().label_selected,
            placeholder_color: self.palette().label_selected,
            background: iced::Background::Color(self.palette().dark_square),
            border_radius: 0.5.into(),
            border_width: 1.,
            border_color: self.palette().dark_square,
            handle_color: self.palette().dark_square,
        }
    }
}

impl menu::StyleSheet for Theme {
    type Style = Menu;

    fn appearance(&self, _style: &Self::Style) -> menu::Appearance {
        menu::Appearance {
            text_color: self.palette().tab_label,
            background: iced::Background::Color(self.palette().light_square),
            border_radius: 0.3.into(),
            border_width: 1.,
            border_color: self.palette().dark_square,
            selected_text_color: self.palette().label_selected,
            selected_background: iced::Background::Color(self.palette().dark_square),
        }
    }
}

impl slider::StyleSheet for Theme {
    type Style = Slider;

    fn active(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (Color::BLACK, Color::BLACK),
                width: 2.0,
                border_radius: 2.0.into(),
            },
            handle: Handle {
                shape: HandleShape::Rectangle {
                    width: 7,
                    border_radius: 1.0.into(),
                },
                color: self.palette().light_square,
                border_width: 2.,
                border_color: self.palette().dark_square,
            },
        }
    }

    fn hovered(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (Color::BLACK, Color::BLACK),
                width: 2.0,
                border_radius: 2.0.into(),
            },
            handle: Handle {
                shape: HandleShape::Rectangle {
                    width: 10,
                    border_radius: 1.0.into(),
                },
                color: self.palette().dark_square,
                border_width: 2.,
                border_color: self.palette().light_square,
            },
        }
    }

    fn dragging(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (Color::BLACK, Color::BLACK),
                width: 2.0,
                border_radius: 2.0.into(),
            },
            handle: Handle {
                shape: HandleShape::Rectangle {
                    width: 10,
                    border_radius: 1.0.into(),
                },
                color: self.palette().dark_square,
                border_width: 2.,
                border_color: self.palette().light_square,
            },
        }
    }
}

impl radio::StyleSheet for Theme {
    type Style = Radio;

    fn active(&self, _style: &Radio, _is_selected: bool) -> radio::Appearance {
        radio::Appearance {
            background: iced::Background::Color(self.palette().light_square),
            dot_color: self.palette().tab_label,
            border_width: 1.,
            border_color: self.palette().dark_square,
            text_color: Some(self.palette().simple_text),
        }
    }

    fn hovered(&self, _style: &Self::Style, _is_selected: bool) -> radio::Appearance {
        radio::Appearance {
            background: iced::Background::Color(self.palette().dark_square),
            dot_color: self.palette().label_selected,
            border_width: 1.,
            border_color: self.palette().simple_text,
            text_color: Some(self.palette().simple_text),
        }
    }
}

/// Offline Chess Puzzles Palette
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OCPPalette {
    pub container_bg: Color,
    pub simple_text: Color,
    pub label_selected: Color,
    pub light_square: Color,
    pub dark_square: Color,
    pub selected_light_square: Color,
    pub selected_dark_square: Color,
    pub tab_label: Color,
}

impl OCPPalette {
    pub const BLUE: Self = Self {
        container_bg: Color::WHITE,
        light_square: rgb!(235.0, 249.0, 255),
        dark_square: rgb!(110.0, 174.0, 213.0),
        selected_light_square: rgb!(205, 210, 106),
        selected_dark_square: rgb!(170, 162, 58),
        simple_text: Color::BLACK,
        label_selected: Color::WHITE,
        tab_label: Color::BLACK,
    };
}

#[derive(Default)]
pub enum ButtonStyle {
    #[default]
    Normal,
    LightSquare,
    DarkSquare,
    SelectedLightSquare,
    SelectedDarkSquare,
}
