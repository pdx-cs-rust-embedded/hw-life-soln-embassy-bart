//! Conway's Game of Life implemented on an `NR`×`NC` "frame
//! buffer" of `u8` pixels that can be either 0 or 1.

/// Compute one row of the Game of Life.
#[allow(clippy::manual_range_contains)]
fn life_row<const NR: usize, const NC: usize>(
    current: &[[u8; NC]; NR],
    next: &mut [[u8; NC]; NR],
    row: usize,
) {
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
        let neighbors: u8 = coords
            .into_iter()
            .map(|(r, c)| {
                let v = current[r][c];
                assert!(v <= 1);
                v
            })
            .sum();
        next[row][col] = match (current[row][col], neighbors) {
            (1, n) if n < 2 || n > 3 => 0,
            (0, 3) => 1,
            (v, _) => v,
        };
    }
}

/// Async version of `life` that yields every `LIFE_CHUNK` rows,
/// allowing the executor to interleave DMA transfers.
pub async fn life_async<const NR: usize, const NC: usize>(
    current: &[[u8; NC]; NR],
    next: &mut [[u8; NC]; NR],
) {
    const LIFE_CHUNK: usize = 10;
    for start in (0..NR).step_by(LIFE_CHUNK) {
        let end = (start + LIFE_CHUNK).min(NR);
        for row in start..end {
            life_row(current, next, row);
        }
        embassy_futures::yield_now().await;
    }
}
