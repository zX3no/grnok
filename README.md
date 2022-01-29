<h1 align="center" style="font-size: 55px">Gronk</h1>

<div align="center" style="display:inline">
      <img src="media/gronk-2x.gif">
</div>

## ✨ Features
- Vim-style key bindings
- Easy to use
- Fuzzy search
- Mouse support
- Cross-platform
- Plays FLAC, WAV, OGG, MP3

## 📦 Installation

> MacOS has not been testing.

> I recommend using a font with ligatures for the best experience.

Debian requires `libasound2-dev`

Fedora requires `alsa-lib-devel`

#### From source

```
git clone https://github.com/zX3no/gronk
cd gronk
cargo install --path gronk
```

Then add some music:
```
gronk add D:/Music
```

## ⌨️ Key Bindings

| Command     | Key       |
|-------------|-----------|
| Move Up     | K / UP    |
| Move Down   | J / Down  |
| Move Left   | H / Left  |
| Move Right  | L / Right |
| Volume Up   | W         |
| Volume Up   | S         |
| Play/Pause  | Space     |
| Previous    | A         |
| Next        | D         |
| Seek -10s   | Q         |
| Seek 10s    | E         |
| Delete Song | X         |
| Clear Queue | C         |
| Change Mode | Tab       |
| Search      | /         |
| ?           | Escape    |
| ?           | Backspace |

## ⚒️ Troubleshooting
> AAC and M4A files seem to be broken ¯\\\_(ツ)_/¯

If somethings goes wrong with the database, you can always delete it using `gronk reset`.

If your music player has broken lines, just increase your zoom level or font size.

![](media/broken.png)

## TODO
- [ ] Fix AAC and M4A files 

- [x] Song metadata (duration)

- [x] Global hotkeys

- [ ] Settings menu (output device, music directories)

- [ ] Replay gain support

- [ ] Automatically adjust margin based on viewable songs

- [ ] Icon

- [ ] Allow the user to click on songs in the queue(partialy working)

- [ ] Toggles for artist/album/song only search

- [ ] Gronk player and UI should be seperate like mpd/client. The player should hold state such as the queue, volume and handle the music output.

## ❤️ Contributing

Feel free to open an issue or submit a pull request!