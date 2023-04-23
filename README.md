<div align="center">
    <img src="https://cdn.discordapp.com/attachments/637119506690474004/1099825353708556318/image.png"/>
</div>

# Requirements

- A redis server, I used [Memurai](https://www.memurai.com/) because I'm on Windows, but you can just use [redis](https://archlinux.org/packages/community/x86_64/redis/) if ur on Arch or something
- Rust nightly toolchain (for now), feel free to make a PR that doesnt require the nightly feature flags im lazy though
- Wiki dumps in SQL from any date (ungzipped), you need 3 tables: `page`, `redirects`, and `pagelinks`
    - If you want to skip the graph building, you can instead just use [this redis dump](https://drive.google.com/file/d/1Fd55I1FJMUXg4VBxnGKJuaqN7z9YCXYg/view) I made from the `2023-04-01` wiki dumps
    - Also if you want to use this on thewikigame, I would use an older wiki dump (like ~2020), [heres a redis dump](https://drive.google.com/file/d/1JtHu2oJISvEFoujb6csyypP4yYBn2OZG/view) I made from `2020-09-20`

# Setup

You can skip this if you're just using a prebuilt redis dump instead
- Set the folder with your wiki dumps and the timestamp of the wiki dumps in the `.env file
- Maybe edit the `THREAD_COUNT` const in the `build` module, I set it to how many cores I have and anything above that seemed to have the same or worse performance.
- Run `cargo run --release -- build` this took me 40 minutes to do.

# Usage examples
- `cargo run --release -- find Kangaroo Coca-Cola` (~50ms)
- `cargo run --release -- find Quantum_Physics Carpenter_bee` (~8s)
- `cargo run --release -- find Hairbrush Everhood` (~400s)
- `pnpm start` run websocket used in `server/wikipedia.user.js` (install npm dependencies first)

# Todo
- Alter the `bfs` so that you can get multi paths somehow
- Maybe make the `bfs` optionally double-sides (optional cus its a bit more cheaty)
