# Camel Up - Bevy 0.17 Implementation Status

## Overview
A digital implementation of the board game **Camel Up (Second Edition)** using Bevy 0.17 with bevy_egui for UI.

## Tech Stack
- **Bevy 0.17** - Game engine (uses `Message`/`MessageReader`/`MessageWriter` instead of Event/EventReader/EventWriter)
- **bevy_egui 0.38.0** - UI framework (uses `EguiPrimaryContextPass` schedule)
- **Rust** - Language

---

## Completed Features

### Phase 1: Project Setup & Basic Rendering ✅
- Bevy 0.17 + bevy_egui initialized
- 2D camera setup
- Race track rendered (16 spaces in oval layout)
- 5 racing camels spawned with colored sprites
- 2 crazy camels spawned (black/white, start on spaces 13-15)

### Phase 2: Core Game State & Components ✅
- `GameState` enum: `MainMenu`, `PlayerSetup`, `Playing`, `LegScoring`, `GameEnd`
- Player resource with money, betting cards, desert tile
- Pyramid with 5 colored dice + 2 crazy camel dice
- LegBettingTile stacks (5/3/2 for each camel color)
- RaceBets for overall winner/loser betting

### Phase 3: Camel Movement System ✅
- Dice rolling (random 1-3)
- Camel movement based on die color and value
- Camel stacking (landing on occupied space = stack on top)
- Desert tile effects:
  - **Oasis**: +1 space forward, land on top
  - **Mirage**: -1 space backward, land underneath (shifts existing camels up)
- Owner earns 1 coin when camel lands on their desert tile

### Phase 4: Turn System & Player Actions ✅
- Turn order (advances after each action)
- **Roll Pyramid Die**: Rolls random die, moves camel, earns 1 coin
- **Take Leg Bet**: Takes top tile from chosen camel's stack
- **Place Desert Tile**: Select space 2-16, choose Oasis or Mirage
- **Place Race Bet**: Bet on overall winner or loser (uses one of 5 camel cards)
- Action validation (can't act twice per turn, validates tile placement rules)

### Phase 5: Leg & Game Scoring ✅
- **Leg end detection**: When all 5 regular dice rolled
- **Leg scoring**:
  - 1st place bets: +5/3/2 based on tile value
  - 2nd place bets: +1
  - Wrong bets: -1
  - Pyramid tile holders: +1 each
- **Leg reset** (Continue button):
  - Resets pyramid dice
  - Resets leg betting tiles to 5/3/2
  - Clears player leg bets
  - Clears player pyramid tiles
  - Increments leg number
  - Returns desert tiles to all players
  - Despawns visual desert tile entities
- **Game end detection**: When camel crosses finish line (space >= 16)
- **Final scoring**:
  - Winner bets: +8/5/3/2/1 in order placed, -1 for wrong
  - Loser bets: +8/5/3/2/1 in order placed, -1 for wrong

### Phase 6: UI Implementation ✅
- **Main Menu**: Start Game, Quit buttons
- **Player Setup Screen**: Configure 2-8 players with names and human/AI toggle
- **In-game HUD**:
  - Top bar: Game title, leg number, dice remaining, last roll, Back to Menu
  - Left panel: Current player info, action buttons
  - Right panel: All players list, camel positions
- **Leg Betting Popup**: Shows available tiles with values
- **Race Betting Popup**: Winner/Loser sections with available cards
- **Desert Tile Popup**: Space selection grid, Oasis/Mirage choice
- **Leg Scoring Screen**: Player standings, Continue button
- **Game End Screen**: Winner announcement, final standings, Play Again/Quit

### Phase 7: Crazy Camel Integration ✅
- Crazy camel dice added to pyramid (7 total dice)
- `DieRollResult` enum distinguishes Regular vs Crazy rolls
- Crazy camels move backwards
- Crazy camels land underneath existing camels
- `CrazyCamelRollResult` message for UI updates
- `LastRoll` enum in UI state handles both roll types

### Additional Features ✅
- **Back to Menu**: Button in HUD, cleans up all game entities
- **GameEntity marker component**: For proper cleanup when leaving Playing state
- **Desert tile visuals**: Spawned as sprites on board (green=oasis, orange=mirage)

---

## Project Structure

```
src/
├── main.rs                    # App entry, plugin setup, system registration
├── components/
│   ├── mod.rs                 # Re-exports
│   ├── board.rs               # BoardSpace, DesertTile, PlacedDesertTiles, GameBoard, TRACK_LENGTH
│   ├── camel.rs               # Camel, CamelColor, CrazyCamel, CrazyCamelColor, BoardPosition
│   ├── player.rs              # Players resource, PlayerInfo
│   ├── betting.rs             # LegBetTile, LegBettingTiles, RaceBets
│   └── dice.rs                # Pyramid, Die, DieRollResult, CrazyCamelDie
├── game/
│   ├── mod.rs
│   ├── state.rs               # GameState enum
│   ├── ai.rs                  # AI decision system, AiDifficulty, AiConfig
│   ├── rules.rs               # Game rules
│   └── scoring.rs             # Score calculation
├── systems/
│   ├── mod.rs
│   ├── setup.rs               # setup_game, cleanup_game, GameEntity
│   ├── movement.rs            # move_camel_system, move_crazy_camel_system, get_leading_camel, etc.
│   ├── turn.rs                # TurnState, action handlers, advance_turn_system, check_leg_end, check_game_end
│   ├── leg.rs                 # calculate_leg_scores, calculate_final_scores, reset_for_new_leg
│   ├── animation.rs           # Movement animations, dice roll effects, particles, fireworks, crown
│   ├── input.rs               # Input handling
│   └── render.rs              # Rendering systems
└── ui/
    ├── mod.rs
    ├── main_menu.rs           # main_menu_ui
    ├── player_setup.rs        # player_setup_ui, PlayerSetupConfig
    ├── hud.rs                 # game_hud_ui, UiState, LastRoll, update_ui_on_roll
    ├── scoring.rs             # leg_scoring_ui, game_end_ui
    ├── betting_panel.rs       # Betting UI panel
    ├── race_betting.rs        # Race betting UI
    ├── pyramid.rs             # Pyramid dice UI
    └── characters.rs          # Character/camel UI elements
```

---

## Key Data Structures

### Messages (Events)
- `MoveCamelEvent { color: CamelColor, spaces: u8 }`
- `MoveCrazyCamelEvent { color: CrazyCamelColor, spaces: u8 }`
- `MovementCompleteEvent { crossed_finish: bool }`
- `TakeLegBetAction { color: CamelColor }`
- `PlaceDesertTileAction { space_index: u8, is_oasis: bool }`
- `RollPyramidAction`
- `PlaceRaceBetAction { color: CamelColor, is_winner_bet: bool }`
- `PyramidRollResult { color: CamelColor, value: u8 }`
- `CrazyCamelRollResult { color: CrazyCamelColor, value: u8 }`
- `LegScoringComplete { scores: Vec<(String, i32)> }`

### Resources
- `GameBoard` - Track space positions
- `Players` - All player info, current player index
- `Pyramid` - Dice state, rolled/available
- `LegBettingTiles` - Available leg bet tiles per color
- `RaceBets` - Winner/loser bet stacks
- `PlacedDesertTiles` - Map of space -> (owner_id, is_oasis)
- `TurnState` - Current player, action_taken, leg_number
- `PlayerLegBetsStore` - Tracks leg bets per player
- `PlayerPyramidTiles` - Tracks pyramid tile counts per player
- `UiState` - Popup visibility, last roll display
- `PlayerSetupConfig` - Player configuration before game starts

### Components
- `Camel { color: CamelColor }`
- `CrazyCamel { color: CrazyCamelColor }`
- `BoardPosition { space_index: u8, stack_position: u8 }`
- `DesertTile { owner_id: u8, is_oasis: bool }`
- `GameEntity` - Marker for cleanup
- `BoardSpace { index: u8 }`
- `CamelSprite` - Marker for camel visuals

---

### Phase 7: AI Players ✅
- AI decision system for non-human players
- `AiDifficulty` enum: Random, Basic, Smart
- Basic AI: Evaluates leg betting based on camel positions, takes 5-value tiles on leaders
- Smart AI: Probability estimation for leg bets, race betting with lead analysis
- Desert tile placement heuristics
- Configurable think delay for visible AI actions

### Phase 8: Visual Polish ✅
- Smooth camel movement animations (hop-by-hop with easing)
- Multi-phase dice roll animations (shake, settle, display, fade)
- Particle effects on dice rolls
- Firework celebration system for game end
- Crown drop animation for winning camel
- Ease-out bounce effects

---

## Remaining Work / Future Enhancements
- Sound effects
- Save/load game state
- Online multiplayer
- Expansion pack rules (photographer, etc.)

---

## Known Issues / Warnings
- Minor unused variable warnings in scoring.rs (idx variables)
- Some struct fields marked as never read (BoardSpace.index, DesertTile fields, CrazyCamelDie.value)

---

## Build & Run
```bash
cargo run
```

## Test Compilation
```bash
cargo check
```
