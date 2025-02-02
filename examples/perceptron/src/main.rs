use rui::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use vger::Color;

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Input {
    value: bool,
}

impl Input {
    fn value(&self) -> f32 {
        if self.value {
            1.0
        } else {
            -1.0
        }
    }

    fn value_bool(&self) -> bool {
        self.value
    }

    fn toggle(&mut self) {
        self.value = !self.value;
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Weight {
    value: f32, // -1.0 to 1.0
}

impl Weight {
    fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Perceptron {
    dimensions: (usize, usize),
    inputs: Vec<Input>,
    weights: Vec<Weight>,
    weight_strength: f32,
}

struct PerceptronApp {
    perceptron: Perceptron,
    mouse_position: Option<LocalPoint>,
    mouse_dragging: bool,
    prev_dragged_input: Option<usize>,
    hovered_input: Option<usize>,
}

fn main() {
    let perceptron = Perceptron {
        dimensions: (4, 4),
        inputs: vec![Input { value: false }; 16],
        weights: vec![Weight { value: 0.0 }; 16],
        weight_strength: 10.0,
    };

    let width = perceptron.dimensions.0;
    let height = perceptron.dimensions.1;
    let cell_size = LocalSize::new(1.0, 1.0);

    // Calculate the origin (top-left) points of each grid cell
    let rects: Vec<LocalRect> = (0..height)
        .flat_map(|row| {
            (0..width).map(move |col| {
                // Calculate the origin position directly without adding half cell size
                let origin_x = col as f32 * cell_size.width;
                let origin_y = row as f32 * cell_size.height;
                let origin = LocalPoint::new(origin_x, origin_y);
                LocalRect::new(origin, cell_size)
            })
        })
        .collect();
    let rects = Arc::new(rects);

    state(
        move || PerceptronApp {
            perceptron: perceptron.clone(),
            mouse_position: None,
            mouse_dragging: false,
            hovered_input: None,
            prev_dragged_input: None,
        },
        move |s, cx| {
            let s_arc = Arc::new(s);
            let rects = rects.clone();

            // Calculate the output value by multiplying the input and weight values
            // Divide the sum by the number of cells to get the average value
            let output = cx[s]
                .perceptron
                .inputs
                .iter()
                .zip(cx[s].perceptron.weights.iter())
                .map(|(input, weight)| {
                    input.value() * weight.value() * cx[s].perceptron.weight_strength
                })
                .sum::<f32>()
                / (width * height) as f32;

            fn feedback(state: &mut PerceptronApp, feedback: f32) {
                let inputs = state.perceptron.inputs.clone();
                state
                    .perceptron
                    .weights
                    .iter_mut()
                    .enumerate()
                    .for_each(|(index, weight)| {
                        let input = inputs[index].value();
                        weight.value += input * feedback;
                    });
            }

            vstack((
                vstack((
                    text("Perceptron").padding(Auto),
                    button("Reset", move |cx| {
                        cx[s].perceptron.inputs.iter_mut().for_each(|input| {
                            input.value = false;
                        });
                        cx[s].perceptron.weights.iter_mut().for_each(|weight| {
                            weight.value = 0.0;
                        });
                    })
                    .padding(Auto),
                    button("save to file", move |cx| {
                        // Save the perceptron to a file
                        let perceptron = &cx[s].perceptron;
                        let perceptron_json = serde_json::to_string(&*perceptron).unwrap();
                        let mut file = File::create("perceptron.json").unwrap();
                        file.write_all(perceptron_json.as_bytes()).unwrap();
                    }),
                    button("Load from file", move |cx| {
                        match File::open("perceptron.json") {
                            Ok(file) => {
                                let perceptron: Perceptron =
                                    serde_json::from_reader(file).expect("Invalid format");
                                cx[s].perceptron = perceptron.clone();
                            }
                            Err(_) => (),
                        };
                    })
                    .padding(Auto),
                    button("delete file", move |_| {
                        std::fs::remove_file("perceptron.json").unwrap();
                    }),
                ))
                .size([200.0, 200.0])
                .padding(Auto),
                spacer(),
                hstack((
                    vstack((
                        hstack((
                            text("Input").padding(Auto),
                            button("Clear", move |cx| {
                                cx[s].perceptron.inputs.iter_mut().for_each(|input| {
                                    input.value = false;
                                });
                            })
                            .padding(Auto),
                        )),
                        canvas(move |cx, canvas_rect, vger| {
                            // Calculate the scaling factor to map local coordinates to global coordinates
                            let scale_x = canvas_rect.width() / width as f32;
                            let scale_y = canvas_rect.height() / height as f32;

                            for (index, local_rect) in rects.iter().enumerate() {
                                // Transform the local rectangle to global coordinates
                                let origin = LocalPoint::new(
                                    local_rect.origin.x * scale_x + canvas_rect.min_x(),
                                    local_rect.origin.y * scale_y + canvas_rect.min_y(),
                                );
                                let size = LocalSize::new(
                                    local_rect.size.width * scale_x,
                                    local_rect.size.height * scale_y,
                                );
                                let global_rect = LocalRect::new(origin, size);

                                // Check if the mouse is hovering over the current cell
                                let is_hovering =
                                    cx[s].mouse_position.map_or(false, |mouse_position| {
                                        global_rect.contains(mouse_position)
                                    });

                                if is_hovering {
                                    cx[s].hovered_input = Some(index);
                                }

                                // Interpolate between dark gray and white based on the input value
                                let input = cx[*s_arc].perceptron.inputs[index];
                                let color = if is_hovering {
                                    if input.value_bool() {
                                        Color::gray(0.8)
                                    } else {
                                        Color::gray(0.2)
                                    }
                                } else {
                                    if input.value_bool() {
                                        Color::gray(1.0)
                                    } else {
                                        Color::gray(0.1)
                                    }
                                };
                                let paint = vger.color_paint(color);

                                // Draw the rectangle using the global coordinates
                                vger.fill_rect(global_rect, 10.0, paint);
                            }

                            if cx[s].mouse_dragging {
                                // Check if prev dragged input is not the same as the current hovered input
                                if cx[s].prev_dragged_input != cx[s].hovered_input {
                                    // Update the input value based on the mouse position
                                    if let Some(hovered_input) = cx[s].hovered_input {
                                        // This means that the user is dragging the input value
                                        // and its value hasnt been reversed yet
                                        cx[s].perceptron.inputs[hovered_input].toggle();
                                        cx[s].prev_dragged_input = cx[s].hovered_input;
                                    }
                                }
                            } else {
                                cx[s].prev_dragged_input = None;
                            }
                        })
                        .drag_p(move |cx, local_position, gesture_state, mouse_button| {
                            match gesture_state {
                                GestureState::Began => {
                                    if mouse_button == Some(MouseButton::Left) {
                                        cx[s].mouse_dragging = true;
                                    }
                                }
                                GestureState::Changed => {
                                    if cx[s].mouse_position.is_some() {
                                        cx[s].mouse_position = Some(local_position);
                                    }
                                }
                                GestureState::Ended => {
                                    cx[s].mouse_dragging = false;
                                    cx[s].hovered_input = None;
                                }
                                #[allow(unreachable_patterns)]
                                _ => (),
                            }
                        })
                        .hover_p(move |cx, hover_position| {
                            cx[s].mouse_position = Some(hover_position);
                        })
                        .hover(move |cx, is_hovering| {
                            if !is_hovering {
                                cx[s].mouse_position = None;
                            }
                        })
                        .size([400.0, 400.0])
                        .padding(Auto),
                    )),
                    vstack((
                        text("Feedback").padding(Auto),
                        button("Positive", move |cx| {
                            feedback(&mut cx[s], 0.01);
                        })
                        .padding(Auto),
                        button("Negative", move |cx| {
                            feedback(&mut cx[s], -0.01);
                        }),
                        spacer(),
                    ))
                    .size([100.0, 400.0])
                    .padding(Auto),
                    vstack((
                        hstack((text("Weights").padding(Auto),)),
                        list((0..height).map(|id| id).collect(), move |x| {
                            let x = *x;
                            hlist((0..width).map(|id| id).collect(), move |y| {
                                let y = *y;
                                with_cx(move |cx| {
                                    let id = x * width + y;
                                    map(
                                        // Map from -1.0 - 1.0 to 0.0 - 1.0
                                        cx[s].perceptron.weights[id].value / 2.0 + 0.5,
                                        // Map from 0.0 - 1.0 to -1.0 - 1.0
                                        move |v, cx| {
                                            cx[s].perceptron.weights[id].value = v * 2.0 - 1.0;
                                        },
                                        |state, _| knob(state).padding(Auto),
                                    )
                                })
                            })
                        })
                        .size([400.0, 400.0])
                        .padding(Auto),
                    )),
                    vstack((
                        hstack((
                            text("Output").padding(Auto),
                            text(format!("{:.2}", output).as_str()).padding(Auto),
                        )),
                        canvas(move |_, canvas_rect, vger| {
                            let red = Color::new(1.0, 0.0, 0.0, 1.0);
                            let green = Color::new(0.0, 1.0, 0.0, 1.0);

                            // is the output greater than 0?
                            let activated = output > 0.0;
                            let paint = vger.color_paint(if activated { green } else { red });

                            // Draw a rectangle in the center of the canvas
                            let size = LocalSize::new(30.0, 30.0);
                            let rect = LocalRect::new(
                                LocalPoint::new(
                                    canvas_rect.width() / 2.0 - size.width / 2.0,
                                    canvas_rect.height() / 2.0 - size.height / 2.0,
                                ),
                                size,
                            );

                            vger.fill_rect(rect, 10.0, paint);
                        })
                        .size([200.0, 400.0])
                        .padding(Auto),
                    )),
                    spacer(),
                )),
            ))
        },
    )
    .run();
}
