use rui::*;
use vger::Color;

fn main() {
    let width = 10;
    let height = 10;
    let grid_width = 10; // Number of columns in the grid
    let grid_height = 10; // Number of rows in the grid

    // Calculate the size of each grid cell
    let cell_width = width / grid_width;
    let cell_height = height / grid_height;
    let cell_size = LocalSize::new(cell_width as f32, cell_height as f32);

    // Calculate the center points of each grid cell
    let rects: Vec<LocalRect> = (0..grid_height)
        .flat_map(|row| {
            (0..grid_width).map(move |col| {
                let center_x = (col * cell_width) + (cell_width / 2);
                let center_y = (row * cell_height) + (cell_height / 2);
                let origin = LocalPoint::new(center_x as f32, center_y as f32);
                LocalRect::new(origin, cell_size)
            })
        })
        .collect();

    canvas(move |_, canvas_rect, vger| {
        let paint = vger.color_paint(Color::gray(0.5));

        // Calculate the scaling factor to map local coordinates to global coordinates
        let scale_x = canvas_rect.width() / width as f32;
        let scale_y = canvas_rect.height() / height as f32;

        for local_rect in &rects {
            // Transform the local rectangle to global coordinates
            let global_rect = LocalRect::new(
                LocalPoint::new(
                    local_rect.origin.x * scale_x + canvas_rect.min_x(),
                    local_rect.origin.y * scale_y + canvas_rect.min_y(),
                ),
                LocalSize::new(
                    local_rect.size.width * scale_x,
                    local_rect.size.height * scale_y,
                ),
            );

            // Draw the rectangle using the global coordinates
            vger.fill_rect(global_rect, 10.0, paint);
        }
    })
    .run()
}
