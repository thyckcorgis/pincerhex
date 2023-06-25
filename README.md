# Pincerhex

## TODOs

### WASM Library

- Create WASM pure function that can generate a move

### Frontend

Create frontend to be able to play on the browser

### Pincerhex Core (nice-to-haves)

- Strip out anything that's not needed and place it behind a feature flag

- TODO: State's move_count is actually wrong, we don't take into account the
  moves that the other player has created

  Makes me think. Do we even need to keep track of `move_count`? Isn't it just
  the number of pieces on the board + 1 if we did a swap move?
