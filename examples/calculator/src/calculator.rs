use enterpolation::{linear::ConstEquidistantLinear, Generator};
use palette::LinSrgb;
use rui::*;

#[derive(PartialEq, Clone, Copy)]
enum ClearState {
    Initial,
    JustCleared,
}

// Enum to represent arithmetic operations.
#[derive(Clone, Copy)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

// Enum to represent special operators like AC, toggle sign, etc.
#[derive(Clone, Copy)]
enum SpecialOperator {
    Clear,      // Clear all/current
    ToggleSign, // Toggle the sign (+/-)
    Percentage, // Percentage operator (%)
    Decimal,    // Decimal point operator (.)
    Equals,     // Equals operator (=)
}

#[derive(Clone)]
enum Button {
    Digit(u64),
    Operator(Operator),
    Special(SpecialOperator),
}

struct ButtonState {
    is_hovered: bool,
    is_touched: bool,
}

#[derive(Clone)]
pub struct CalculatorConfig {
    dark_mode: bool,
    rounded_corners: bool,
}

impl CalculatorConfig {
    pub fn dark_mode(mut self) -> Self {
        self.dark_mode = true;
        self
    }

    pub fn rounded_corners(mut self) -> Self {
        self.rounded_corners = true;
        self
    }

    pub fn show(self) -> impl View {
        Calculator::with_config(&self).show()
    }
}

#[derive(Clone)]
pub struct Calculator {
    ocean: enterpolation::linear::Linear<
        enterpolation::ConstEquidistant<f32, 2>,
        [palette::rgb::Rgb<palette::encoding::Linear<palette::encoding::Srgb>>; 2],
        enterpolation::Identity,
    >,
    sky: enterpolation::linear::Linear<
        enterpolation::ConstEquidistant<f32, 2>,
        [palette::rgb::Rgb<palette::encoding::Linear<palette::encoding::Srgb>>; 2],
        enterpolation::Identity,
    >,
    text_color: vger::Color,
    background_color: vger::Color,
    background_corner_radius: f32,
    number_display_color: vger::Color,
}

impl Calculator {
    pub fn new() -> CalculatorConfig {
        CalculatorConfig {
            dark_mode: false,
            rounded_corners: false,
        }
    }

    pub fn with_config(config: &CalculatorConfig) -> Calculator {
        let ocean_colors = if config.dark_mode {
            [LinSrgb::new(0.05, 0.2, 0.40), LinSrgb::new(0.1, 0.25, 0.30)]
        } else {
            [LinSrgb::new(0.05, 0.2, 0.40), LinSrgb::new(0.1, 0.25, 0.30)]
        };

        let ocean = ConstEquidistantLinear::<f32, _, 2>::equidistant_unchecked(ocean_colors);

        let sky_colors = if config.dark_mode {
            [
                LinSrgb::new(0.00, 0.25, 0.35),
                LinSrgb::new(0.05, 0.20, 0.40),
            ]
        } else {
            [
                LinSrgb::new(0.00, 0.25, 0.35),
                LinSrgb::new(0.05, 0.20, 0.40),
            ]
        };

        let sky = ConstEquidistantLinear::<f32, _, 2>::equidistant_unchecked(sky_colors);

        let text_color = if config.dark_mode {
            vger::Color::new(1.0, 1.0, 1.0, 1.0)
        } else {
            vger::Color::new(0.0, 0.0, 0.0, 1.0)
        };

        let background_color = if config.dark_mode {
            vger::Color::new(0.1, 0.1, 0.1, 1.0)
        } else {
            vger::Color::new(1.0, 1.0, 1.0, 1.0)
        };

        let number_display_color = if config.dark_mode {
            vger::Color::new(0.2, 0.2, 0.2, 1.0)
        } else {
            vger::Color::new(0.2, 0.2, 0.2, 1.0)
        };

        let background_corner_radius = if config.rounded_corners { 10.0 } else { 0.0 };

        Calculator {
            ocean,
            sky,
            text_color,
            background_color,
            background_corner_radius,
            number_display_color,
        }
    }

    fn show(self) -> impl View {
        zstack((
            rectangle()
                .color(self.background_color)
                .corner_radius(self.background_corner_radius),
            state(
                move || CalculatorState::new(),
                move |s: StateHandle<CalculatorState>, cx: &Context| {
                    vstack((
                        self.display_value(&cx[s]),
                        hstack((
                            self.button_view(s, Button::Special(SpecialOperator::Clear), 0),
                            self.button_view(s, Button::Special(SpecialOperator::ToggleSign), 1),
                            self.button_view(s, Button::Special(SpecialOperator::Percentage), 2),
                            self.button_view(s, Button::Operator(Operator::Divide), 3),
                        )),
                        hstack((
                            self.button_view(s, Button::Digit(7), 4),
                            self.button_view(s, Button::Digit(8), 5),
                            self.button_view(s, Button::Digit(9), 6),
                            self.button_view(s, Button::Operator(Operator::Multiply), 7),
                        )),
                        hstack((
                            self.button_view(s, Button::Digit(4), 8),
                            self.button_view(s, Button::Digit(5), 9),
                            self.button_view(s, Button::Digit(6), 10),
                            self.button_view(s, Button::Operator(Operator::Subtract), 11),
                        )),
                        hstack((
                            self.button_view(s, Button::Digit(1), 12),
                            self.button_view(s, Button::Digit(2), 13),
                            self.button_view(s, Button::Digit(3), 14),
                            self.button_view(s, Button::Operator(Operator::Add), 15),
                        )),
                        hstack((
                            self.button_view(s, Button::Digit(0), 16),
                            self.button_view(s, Button::Special(SpecialOperator::Decimal), 17),
                            self.button_view(s, Button::Special(SpecialOperator::Equals), 18),
                        )),
                    ))
                    .padding(Auto)
                },
            ),
        ))
    }

    fn display_value(&self, state: &CalculatorState) -> impl View {
        let display_text = if state.has_error {
            "Error".to_string()
        } else if state.second_operand.is_empty() {
            "0".to_string()
        } else {
            state.second_operand.clone()
        };
        zstack((
            rectangle()
                .corner_radius(10.0)
                .color(self.number_display_color),
            text(&display_text)
                .font_size(40)
                .size([0.0, 50.0])
                .offset([10.0, 10.0]),
        ))
        .padding(Auto)

        // canvas(move |_, rect, vger| {
        //     vger.save();
        //     let color = vger::Color::new(0.2, 0.2, 0.2, 1.0);
        //     let paint_index = vger.color_paint(color);
        //     vger.fill_rect(rect, 10.0, paint_index);

        //     vger.restore();
        //     vger.save();

        //     let text_height: u32 = 40;

        //     let origin = vger.text_bounds(&display_text, text_height, None).origin;

        //     vger.translate([
        //         -origin.x,
        //         -origin.y + rect.height() / 2.0 - (text_height as f32) / 2.0,
        //     ]);

        //     let text_color = vger::Color::new(1.0, 1.0, 1.0, 1.0);

        //     vger.text(&display_text, text_height, text_color, Some(0.0));
        //     vger.restore();
        // })
    }

    fn button_view(&self, s: StateHandle<CalculatorState>, button: Button, id: usize) -> impl View {
        let calculator = self.clone();

        let is_number = matches!(button, Button::Digit(_));
        let digit = match button {
            Button::Digit(d) => d,
            _ => 0,
        };

        state(
            || ButtonState {
                is_hovered: false,
                is_touched: false,
            },
            move |button_state, cx: &Context| {
                let button_clone = button.clone();

                let alpha = if cx[button_state].is_touched {
                    0.5
                } else if cx[button_state].is_hovered {
                    0.8
                } else {
                    1.0
                };
                let color = {
                    let color = if is_number {
                        // cx[s].blue_gradient.gen(id as f32 / 18.0)
                        calculator.ocean.gen(digit as f32 / 9.0)
                    } else {
                        calculator.sky.gen(id as f32 / 18.0)
                    };
                    vger::Color::new(color.red, color.green, color.blue, alpha)
                };

                zstack((
                    rectangle()
                        .corner_radius(10.0)
                        .color(color)
                        .touch(move |cx, info| match info.state {
                            TouchState::Begin => {
                                cx[button_state].is_touched = true;
                            }
                            TouchState::End => {
                                cx[button_state].is_touched = false;
                                cx[s].button_action(button_clone.clone());
                            }
                        })
                        .hover(move |cx, hovered| {
                            cx[button_state].is_hovered = hovered;
                        }),
                    {
                        let label = match button {
                            Button::Digit(digit) => digit.to_string(),
                            Button::Operator(op) => match op {
                                Operator::Add => "+".to_string(),
                                Operator::Subtract => "-".to_string(),
                                Operator::Multiply => "*".to_string(),
                                Operator::Divide => "/".to_string(),
                            },
                            Button::Special(action) => match action {
                                SpecialOperator::Clear => {
                                    if cx[s].clear_state == ClearState::JustCleared {
                                        "AC".to_string()
                                    } else {
                                        "C".to_string()
                                    }
                                }
                                SpecialOperator::ToggleSign => "+/-".to_string(),
                                SpecialOperator::Percentage => "%".to_string(),
                                SpecialOperator::Equals => "=".to_string(),
                                SpecialOperator::Decimal => ".".to_string(),
                            },
                        };

                        text(&label)
                    }
                    .font_size(30)
                    .color(calculator.text_color)
                    .offset([10.0, 10.0]),
                ))
                .padding(Auto)
            },
        )
    }
}

// Struct to represent the calculator's state.
pub struct CalculatorState {
    first_operand: String,
    second_operand: String,
    current_operator: Option<Operator>,
    is_input_new: bool,
    has_error: bool,
    is_result_displayed: bool,
    last_operator: Option<Operator>,
    clear_state: ClearState,
}

impl CalculatorState {
    fn new() -> Self {
        Self {
            first_operand: "0".to_string(),
            second_operand: "0".to_string(),
            current_operator: None,
            is_input_new: true,
            has_error: false,
            is_result_displayed: false,
            last_operator: None,
            clear_state: ClearState::Initial,
        }
    }

    fn execute_operation(&mut self) {
        if self.has_error {
            return;
        }

        let first_operand: f64 = self.first_operand.parse().unwrap_or(0.0);
        let second_operand: f64 = self.second_operand.parse().unwrap_or(0.0);

        let result = match self.current_operator {
            Some(Operator::Add) => first_operand + second_operand,
            Some(Operator::Subtract) => first_operand - second_operand,
            Some(Operator::Multiply) => first_operand * second_operand,
            Some(Operator::Divide) => {
                if second_operand == 0.0 {
                    self.has_error = true;
                    return; // Early return on division by zero
                }
                first_operand / second_operand
            }
            None => return, // Handle the case where no operator is set
        };

        self.second_operand = result.to_string();
        self.first_operand = self.second_operand.clone();
        self.is_input_new = true;
        self.is_result_displayed = true;
        self.current_operator = None;
    }

    fn input_digit(&mut self, digit: u64) {
        if self.is_result_displayed {
            self.second_operand = String::new(); // Clear on new input after result
            self.is_result_displayed = false;
        }

        if self.second_operand == "0" && digit == 0 && !self.second_operand.contains('.') {
            return; // Prevent multiple leading zeros before decimal
        }
        if self.second_operand == "0" && !self.second_operand.contains('.') {
            self.second_operand = String::new();
        }

        self.second_operand.push_str(&digit.to_string());
    }

    fn input_decimal(&mut self) {
        if self.is_result_displayed {
            self.second_operand = "0.".to_string(); // Start with "0." after result
            self.is_result_displayed = false;
            return;
        }
        if !self.second_operand.contains('.') {
            if self.second_operand.is_empty() {
                self.second_operand.push_str("0.");
            } else {
                self.second_operand.push('.');
            }
        }
    }

    fn toggle_sign(&mut self) {
        if !self.second_operand.is_empty() && self.second_operand != "0" {
            if self.second_operand.starts_with('-') {
                self.second_operand = self.second_operand[1..].to_string();
            } else {
                self.second_operand = format!("-{}", self.second_operand);
            }
        }
    }

    fn apply_percentage(&mut self) {
        if let Ok(value) = self.second_operand.parse::<f64>() {
            self.second_operand = (value / 100.0).to_string();
        }
    }

    fn reset(&mut self) {
        self.first_operand = "0".to_string();
        self.second_operand = "0".to_string();
        self.current_operator = None;
        self.last_operator = None;
        self.is_input_new = true;
        self.has_error = false;
        self.is_result_displayed = false;
        self.clear_state = ClearState::Initial; // Important: Reset clear_state
    }

    fn button_action(&mut self, button: Button) {
        match button {
            Button::Digit(_)
            | Button::Operator(_)
            | Button::Special(SpecialOperator::ToggleSign)
            | Button::Special(SpecialOperator::Percentage)
            | Button::Special(SpecialOperator::Equals)
            | Button::Special(SpecialOperator::Decimal) => {
                if self.clear_state == ClearState::JustCleared {
                    self.reset();
                } else {
                    self.clear_state = ClearState::Initial;
                }

                match button {
                    Button::Digit(digit) => self.input_digit(digit),
                    Button::Operator(op) => {
                        if !self.second_operand.is_empty() {
                            if self.current_operator.is_some() {
                                self.execute_operation();
                            } else {
                                self.first_operand = self.second_operand.clone();
                                self.second_operand = "0".to_string();
                            }
                        }
                        self.current_operator = Some(op);
                        self.is_input_new = true;
                        self.is_result_displayed = false;
                    }
                    Button::Special(action) => match action {
                        SpecialOperator::ToggleSign => self.toggle_sign(),
                        SpecialOperator::Percentage => self.apply_percentage(),
                        SpecialOperator::Equals => {
                            if self.current_operator.is_some() && !self.second_operand.is_empty() {
                                self.execute_operation();
                            }
                        }
                        SpecialOperator::Decimal => self.input_decimal(),
                        _ => (),
                    },
                }
            }
            Button::Special(SpecialOperator::Clear) => {
                if self.second_operand == "0"
                    && self.first_operand == "0"
                    && self.current_operator.is_none()
                {
                    self.reset();
                    self.clear_state = ClearState::JustCleared;
                } else {
                    self.second_operand = "0".to_string();
                    self.is_input_new = true;
                    self.clear_state = ClearState::Initial;
                }
            }
        }
    }
}
