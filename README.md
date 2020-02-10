# Bookworm 🐛

Bookworm is a [BattleSnake][1] bot written in [Rust][2] for the March 2020 tournament. Combines the strategies of pruned turn tree exploration, minimax, and heuristic scoring. The snake is given a time budget to explore the turn tree; the fewer the snakes, the deeper the exploration.

![](screenshot.png)
_A server instance playing against itself using the built-in host mode. See `play-host.sh` as an example._

## Building and running

Just run `cargo build --release` to produce a self-contained binary at `target/release/bookworm`. The binary can be invoked with a number of modes and options, which the `-h` flag explains in detail. The available modes are:

* **server:** Runs as a typical snake API server, ready to be play.
* **host:** Locally hosts a match between given snakes, logging each turn state. Implements 2020 rules.
* **benchmark:** A series of common operations are timed and logged.

## Development

Unit tests can be run with `cargo test`, though some strategy tests will fail currently. To quickly build and run the bot, use `cargo run <mode>`. Note that the development build is significantly slower at runtime than the release build, so you may need to increase the `--timeout` for host mode and give the server more time budget with `--budget` to achieve similar lookahead depths.

#### Todos and improvement ideas:
* Implement more unit tests for behaviour; fix the failing ones
* Look for opportunities to use [infallible DS][3] to avoid empty checks
* Seed the turn tree exploration with some longer term "plays" instead of just single space movements
* Move hosting to us-west-1?

## Resources

* https://docs.battlesnake.com/rules
* https://github.com/BattlesnakeOfficial/rules/blob/master/standard.go
* https://github.com/BattlesnakeOfficial/engine/blob/master/rules/tick.go
* https://github.com/anvaka/ngraph.path
* https://github.com/riscy/a_star_on_grids
* https://www.redblobgames.com/pathfinding/grids/algorithms.html
* [http://likebike.com/posts/How_To_Write_Fast_Rust_Code.html][3]

[1]: https://play.battlesnake.com/
[2]: https://www.rust-lang.org/
[3]: http://likebike.com/posts/How_To_Write_Fast_Rust_Code.html
