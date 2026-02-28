//! Conway's Game of Life implemented on a 5×5 "frame
//! buffer" of `u8` pixels that can be either 0 or 1.

/// Make a step according to the Game of Life rules.
pub fn life<const NR: usize, const NC: usize>(fb: &mut [[u8; NC]; NR]) {
    let prev = *fb;
    for row in 0..NR {
        for col in 0..NC {
            let prev_row = (row + NR - 1) % NR;
            let next_row = (row + 1) % NR;
            let prev_col = (col + NC - 1) % NC;
            let next_col = (col + 1) % NC;
            let coords = [
                (prev_row, prev_col),
                (prev_row, col),
                (prev_row, next_col),
                (row, prev_col),
                (row, next_col),
                (next_row, prev_col),
                (next_row, col),
                (next_row, next_col),
            ];
            let neighbors = coords
                .into_iter()
                .map(|(r, c)| {
                    let v = prev[r][c];
                    assert!(v <= 1);
                    v
                })
                .sum();
            #[allow(clippy::manual_range_contains)]
            match (prev[row][col], neighbors) {
                (1, n) if n < 2 || n > 3 => fb[row][col] = 0,
                (0, 3) => fb[row][col] = 1,
                (_, _) => (),
            }
        }
    }
}
