use enterpolation::{linear::ConstEquidistantLinear, Curve};
use palette::LinSrgb;
use rui::vger::Color;
use rui::*;
use std::sync::Arc;

#[derive(Clone)]
struct FlipCard {
    question: Arc<str>,
    answer: Arc<str>,
    color: Color,
}

enum Action {
    PreviousCard,
    NextCard,
    ToggleAnswer,
}

#[derive(Default)]
struct FlipCardAnimation {
    entrance_offset: Option<LocalOffset>,
    entrance_velocity: Option<LocalOffset>,
    offset: Option<LocalOffset>,
    animated_offset: Option<LocalOffset>,
    animated_offset_velocity: Option<LocalOffset>,
    action: Option<Action>,
    show_answer: bool,
    card_id: usize,
}

impl FlipCardAnimation {
    fn new() -> Self {
        Self {
            entrance_offset: Some(LocalOffset::new(0.0, 1000.0)),
            ..Default::default()
        }
    }
}

fn flip_cards_view(v: Vec<FlipCard>) -> impl View {
    state(FlipCardAnimation::new, move |a, cx| {
        let text_title = if cx[a].show_answer {
            "Answer"
        } else {
            "Question"
        };

        let data = if cx[a].show_answer {
            &v[cx[a].card_id].answer
        } else {
            &v[cx[a].card_id].question
        };

        let flip_cards_count = v.len();

        let base_color = v[cx[a].card_id].color;
        let text_color = if base_color.r + base_color.g + base_color.b > 1.5 {
            Color::gray(0.0)
        } else {
            Color::gray(1.0)
        };

        zstack((
            rectangle()
                .color(v[cx[a].card_id].color)
                .corner_radius(15.0),
            hstack((
                spacer(),
                vstack((
                    spacer(),
                    vstack((
                        text(text_title)
                            .font_size(20)
                            .color(text_color)
                            .padding(Auto),
                        text(data).font_size(15).color(text_color).padding(Auto),
                    )),
                    spacer(),
                    text("tap to flip")
                        .font_size(13)
                        .color(text_color)
                        .padding(Auto),
                    spacer().size([0.0, 20.0]),
                )),
                spacer(),
            )),
        ))
        .offset(
            cx[a].animated_offset.unwrap_or_default() + cx[a].entrance_offset.unwrap_or_default(),
        )
        .size([350.0, 200.0])
        .drag(move |cx, offset, state, _| {
            // Handle drag events
            match state {
                GestureState::Began => {
                    cx[a].offset = Some(offset + cx[a].animated_offset.unwrap_or_default());
                }
                GestureState::Changed => {
                    cx[a].offset = cx[a].offset.map(|o| o + offset);
                }
                GestureState::Ended => {
                    if let Some(cx_offset) = cx[a].offset {
                        let is_horizontal = cx_offset.y.abs() < cx_offset.x.abs();
                        if is_horizontal
                            && (cx_offset.x.abs() > 200.0
                                || cx[a].animated_offset_velocity.unwrap_or_default().x.abs()
                                    > 100.0)
                        {
                            cx[a].action = Some(if cx_offset.x > 0.0 {
                                Action::NextCard
                            } else {
                                Action::PreviousCard
                            });
                        }
                    }

                    cx[a].offset = None;

                    // On tap, toggle the answer visibility
                    if cx[a].animated_offset_velocity.unwrap_or_default().x.abs() < 10.0
                        && cx[a].animated_offset_velocity.unwrap_or_default().y.abs() < 10.0
                    {
                        cx[a].action = Some(Action::ToggleAnswer);
                    }
                }
            }
        })
        .anim(move |cx, dt| {
            // Animate cx[a].entrance_offset towards zero using a spring animation
            let target_offset = LocalOffset::zero();
            let diff = target_offset - cx[a].entrance_offset.unwrap_or_default();
            let speed = 95.0;
            let damping = 11.0;
            let new_velocity = cx[a].entrance_velocity.unwrap_or_default() + diff * speed * dt
                - cx[a].entrance_velocity.unwrap_or_default() * damping * dt;
            cx[a].entrance_velocity = Some(new_velocity);
            let new_offset = cx[a].entrance_offset.unwrap_or_default()
                + cx[a].entrance_velocity.unwrap_or_default() * dt;
            cx[a].entrance_offset = Some(new_offset);

            // Animate animated_offset towards offset
            if let Some(offset) = cx[a].offset {
                let target_offset = offset;
                let animated_offset = cx[a].animated_offset.unwrap_or_default();
                let diff = target_offset - animated_offset;
                let new_velocity = diff * 10.0;
                cx[a].animated_offset_velocity = Some(new_velocity);
                let new_offset =
                    animated_offset + cx[a].animated_offset_velocity.unwrap_or_default() * dt;
                cx[a].animated_offset = Some(new_offset);
            }

            // Continue using the velocity to animate the offset
            // if the user is not dragging the card
            let target_offset = if let Some(exit) = &cx[a].action {
                match exit {
                    Action::PreviousCard => LocalOffset::new(-1400.0, 0.0),
                    Action::NextCard => LocalOffset::new(1400.0, 0.0),
                    Action::ToggleAnswer => LocalOffset::new(0.0, 200.0),
                }
            } else {
                LocalOffset::zero()
            };

            let diff = target_offset - cx[a].animated_offset.unwrap_or_default();

            let is_horizontal = cx[a].animated_offset.unwrap_or_default().y.abs()
                < cx[a].animated_offset.unwrap_or_default().x.abs();

            // If the x offset is far enough, perform the horizontal swipe action
            if is_horizontal && cx[a].animated_offset.unwrap_or_default().x.abs() > 1000.0 {
                match &cx[a].action {
                    Some(Action::PreviousCard) => {
                        cx[a].show_answer = false;
                        cx[a].action = None;
                        cx[a].card_id = (cx[a].card_id + flip_cards_count - 1) % flip_cards_count;
                        cx[a].animated_offset = Some(LocalOffset::new(500.0, 0.0));
                    }
                    Some(Action::NextCard) => {
                        cx[a].show_answer = false;
                        cx[a].action = None;
                        cx[a].card_id = (cx[a].card_id + 1) % flip_cards_count;
                        cx[a].animated_offset = Some(LocalOffset::new(-500.0, 0.0));
                    }
                    _ => {}
                }
            }

            // Perform the show answer action
            if !is_horizontal && cx[a].animated_offset.unwrap_or_default().y > 40.0 {
                match &cx[a].action {
                    Some(Action::ToggleAnswer) => {
                        cx[a].show_answer = !cx[a].show_answer;
                        cx[a].action = None;
                    }
                    _ => {}
                }
            }

            let speed = 105.0;
            let new_velocity =
                cx[a].animated_offset_velocity.unwrap_or_default() + diff * speed * dt;
            cx[a].animated_offset_velocity = Some(new_velocity);

            // Decay the velocity
            let decay_speed = 10.0;
            cx[a].animated_offset_velocity = cx[a]
                .animated_offset_velocity
                .map(|v| v * (1.0 - decay_speed * dt));
            cx[a].animated_offset = cx[a]
                .animated_offset
                .map(|o| o + cx[a].animated_offset_velocity.unwrap_or_default() * dt);
        })
    })
}

// Main function
fn main() {
    let gradient = ConstEquidistantLinear::<f32, _, 2>::equidistant_unchecked([
        LinSrgb::new(0.70, 0.10, 0.20),
        LinSrgb::new(0.95, 0.90, 0.30),
    ]);

    let taken_colors: Vec<_> = gradient.take(10).collect();
    let colors = Arc::new(
        taken_colors
            .iter()
            .map(|c| Color::new(c.red, c.green, c.blue, 1.0))
            .collect::<Vec<_>>(),
    );

    // Search for flip card JSON data
    let mut search: Vec<String> = SearchBuilder::default()
        .location(".")
        .search_input("flip_card_data")
        .ext("json")
        .depth(10)
        .build()
        .collect();

    // Read the JSON file containing the flip cards data
    let data = fs::read_to_string(search.pop().unwrap()).expect("Failed to read data file");

    // Deserialize the raw string into a serde_json::Value
    let value: Value = serde_json::from_str(&data).expect("Failed to parse JSON");

    // Find the flip cards by iteratively searching the JSON
    let flip_cards = find_flip_cards(&value);

    let flip_cards: Vec<FlipCard> = flip_cards
        .iter()
        .enumerate()
        .map(|(i, card)| FlipCard {
            question: card["q"].to_string().into(),
            answer: card["a"].to_string().into(),
            color: colors[i % colors.len()].clone(),
        })
        .collect();

    zstack((
        rectangle().color(Color::gray(0.9)),
        hstack((
            spacer().size([20.0, 0.0]),
            vstack((
                spacer(),
                text("Swipe left to toggle show anwswer")
                    .color(Color::gray(0.0))
                    .font_size(15)
                    .padding(Auto),
                spacer(),
            )),
            spacer(),
            vstack((
                spacer(),
                text("Swipe right for the next card")
                    .color(Color::gray(0.0))
                    .font_size(15)
                    .padding(Auto),
                spacer(),
            )),
            spacer().size([20.0, 0.0]),
        )),
        hstack((
            spacer(),
            vstack((spacer(), flip_cards_view(flip_cards), spacer())),
            spacer(),
        )),
    ))
    .run();
}

use rust_search::SearchBuilder;
use serde_json::{Map, Value};
use std::collections::{HashSet, VecDeque};
use std::fs;

// Function to find and extract flip cards from JSON data
fn find_flip_cards(value: &Value) -> Vec<Value> {
    let mut flip_cards = Vec::new();

    // Define sets of possible keywords for "question" and "answer"
    let question_keywords: HashSet<&str> = ["question", "q", "frontside"].iter().cloned().collect();
    let answer_keywords: HashSet<&str> = ["answer", "a", "backside", "explanation", "solution"]
        .iter()
        .cloned()
        .collect();

    // Use a stack (VecDeque) to mimic recursion
    let mut stack = VecDeque::new();
    stack.push_back(value);

    // While there are items on the stack, process them
    while let Some(current_value) = stack.pop_front() {
        match current_value {
            Value::Object(map) => {
                let mut question = None;
                let mut answer = None;

                for (key, val) in map {
                    // Check if the key matches any question keyword
                    if question_keywords.contains(&key.to_lowercase().as_str()) {
                        if let Some(q) = val.as_str() {
                            question = Some(q.to_string());
                        }
                    }

                    // Check if the key matches any answer keyword
                    if answer_keywords.contains(&key.to_lowercase().as_str()) {
                        if let Some(a) = val.as_str() {
                            answer = Some(a.to_string());
                        }
                    }

                    // Add the value to the stack if it is an object or array (to process nested structures)
                    if val.is_object() || val.is_array() {
                        stack.push_back(val);
                    }
                }

                // If both question and answer were found, add the flip card
                if let (Some(q), Some(a)) = (question, answer) {
                    flip_cards.push(Value::Object({
                        let mut map = Map::with_capacity(2);
                        map.insert("q".to_string(), Value::String(q));
                        map.insert("a".to_string(), Value::String(a));
                        map
                    }));
                }
            }
            Value::Array(arr) => {
                // If the current value is an array, push its elements to the stack
                for item in arr {
                    stack.push_back(item);
                }
            }
            _ => {}
        }
    }

    flip_cards
}
