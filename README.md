# BookWorm 🐛

BookWorm is a [BattleSnake][1] bot written in [Rust][2] for the March 2020 tournament. It aims to combine pruned turn tree exploration with heuristic scoring. It's still a WIP, so will do some dumb things!

![](screenshot.png)
_A server instance playing against itself using the built-in host mode. See `play-self.sh` as an example._

## Building and running

Just run `cargo build --release` to produce a self-contained binary at `target/release/bookworm`. The binary can be invoked with a number of modes and options, which the `-h` flag explains in detail. The available modes are:

* **server:** Runs as a typical snake API server, ready to be play.
* **host:** Locally hosts a match between given snakes, logging each turn state. Implements 2020 rules.
* **benchmark:** A series of common operations are timed and logged.

## Development

Unit tests can be run with `cargo test`, though some strategy tests will fail currently. To quickly build and run the bot, use `cargo run <mode>`. Note that the development build is significantly slower at runtime than the release build, so you may need to increase the `--timeout` for host mode.

#### Todos and improvement ideas:
* Troubleshoot snake's tendency to die in head-on collisions
* Implement more unit tests for behaviour; fix the failing ones
* Self-monitor performance and latency, dynamically tune lookahead depth to best utilize 500 ms allowance
* Maintain bounding rectangles for each snake to speed up hit/free move checks
* Supposedly `if let Some(x) = vec.get(i)` is faster than `vec[i]`?
* Look for opportunities to use [infallible DS][3] to avoid empty checks
* Use Jump Point Search to improve performance of pathfinding
* Improve these hot paths from profiling:
  * Making hashes
  * Pathfinding
  * Min by key

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
