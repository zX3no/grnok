#![allow(unused)]
use browser::Browser;
use gonk_core::{vdb::Database, *};
use gonk_player::{Player, Wasapi};
use once_cell::sync::Lazy;
use playlist::{Mode as PlaylistMode, Playlist};
use queue::Queue;
use search::{Mode as SearchMode, Search};
use settings::Settings;
use std::{
    error::Error,
    fs,
    io::{stdout, Stdout},
    path::Path,
    ptr::addr_of_mut,
    time::{Duration, Instant},
};
use winter::*;

mod browser;
mod playlist;
mod queue;
mod search;
mod settings;

const NUMBER: Color = Color::Green;
const TITLE: Color = Color::Cyan;
const ALBUM: Color = Color::Magenta;
const ARTIST: Color = Color::Blue;
const SEEKER: Color = Color::White;

// static HELP: Lazy<Vec<Row>> = Lazy::new(|| {
//     vec![
//         Row::new(vec![
//             Cell::from("Move Up").style(Style::default().fg(Color::Cyan)),
//             Cell::from("K / UP"),
//         ]),
//         Row::new(vec![
//             Cell::from("Move Down").style(Style::default().fg(Color::Cyan)),
//             Cell::from("J / Down"),
//         ]),
//         Row::new(vec![
//             Cell::from("Move Left").style(Style::default().fg(Color::Cyan)),
//             Cell::from("H / Left"),
//         ]),
//         Row::new(vec![
//             Cell::from("Move Right").style(Style::default().fg(Color::Cyan)),
//             Cell::from("L / Right"),
//         ]),
//         Row::new(vec![
//             Cell::from("Volume Up").style(Style::default().fg(Color::Green)),
//             Cell::from("W"),
//         ]),
//         Row::new(vec![
//             Cell::from("Volume Down").style(Style::default().fg(Color::Green)),
//             Cell::from("S"),
//         ]),
//         Row::new(vec![
//             Cell::from("Mute").style(Style::default().fg(Color::Green)),
//             Cell::from("Z"),
//         ]),
//         Row::new(vec![
//             Cell::from("Play/Pause").style(Style::default().fg(Color::Magenta)),
//             Cell::from("Space"),
//         ]),
//         Row::new(vec![
//             Cell::from("Previous").style(Style::default().fg(Color::Magenta)),
//             Cell::from("A"),
//         ]),
//         Row::new(vec![
//             Cell::from("Next").style(Style::default().fg(Color::Magenta)),
//             Cell::from("D"),
//         ]),
//         Row::new(vec![
//             Cell::from("Seek -10s").style(Style::default().fg(Color::Magenta)),
//             Cell::from("Q"),
//         ]),
//         Row::new(vec![
//             Cell::from("Seek 10s").style(Style::default().fg(Color::Magenta)),
//             Cell::from("E"),
//         ]),
//         Row::new(vec![
//             Cell::from("Queue").style(Style::default().fg(Color::Blue)),
//             Cell::from("1"),
//         ]),
//         Row::new(vec![
//             Cell::from("Browser").style(Style::default().fg(Color::Blue)),
//             Cell::from("2"),
//         ]),
//         Row::new(vec![
//             Cell::from("Playlists").style(Style::default().fg(Color::Blue)),
//             Cell::from("3"),
//         ]),
//         Row::new(vec![
//             Cell::from("Settings").style(Style::default().fg(Color::Blue)),
//             Cell::from("4"),
//         ]),
//         Row::new(vec![
//             Cell::from("Search").style(Style::default().fg(Color::Blue)),
//             Cell::from("/"),
//         ]),
//         Row::new(vec![
//             Cell::from("Exit Search").style(Style::default().fg(Color::Blue)),
//             Cell::from("Escape"),
//         ]),
//         Row::new(vec![
//             Cell::from("Add song to queue").style(Style::default().fg(Color::Cyan)),
//             Cell::from("Enter"),
//         ]),
//         Row::new(vec![
//             Cell::from("Add song to playlist").style(Style::default().fg(Color::Cyan)),
//             Cell::from("Shift + Enter"),
//         ]),
//         Row::new(vec![
//             Cell::from("Move song margin").style(Style::default().fg(Color::Green)),
//             Cell::from("F1 / Shift + F1"),
//         ]),
//         Row::new(vec![
//             Cell::from("Move album margin").style(Style::default().fg(Color::Green)),
//             Cell::from("F2 / Shift + F2"),
//         ]),
//         Row::new(vec![
//             Cell::from("Move artist margin").style(Style::default().fg(Color::Green)),
//             Cell::from("F3 / Shift + F3"),
//         ]),
//         Row::new(vec![
//             Cell::from("Update database").style(Style::default().fg(Color::Yellow)),
//             Cell::from("U"),
//         ]),
//         Row::new(vec![
//             Cell::from("Quit player").style(Style::default().fg(Color::Yellow)),
//             Cell::from("Ctrl + C"),
//         ]),
//         Row::new(vec![
//             Cell::from("Clear queue").style(Style::default().fg(Color::Red)),
//             Cell::from("C"),
//         ]),
//         Row::new(vec![
//             Cell::from("Clear except playing").style(Style::default().fg(Color::Red)),
//             Cell::from("Shift + C"),
//         ]),
//         Row::new(vec![
//             Cell::from("Delete song/playlist").style(Style::default().fg(Color::Red)),
//             Cell::from("X"),
//         ]),
//         Row::new(vec![
//             Cell::from("Delete without confirmation").style(Style::default().fg(Color::Red)),
//             Cell::from("Shift + X"),
//         ]),
//     ]
// });

#[derive(PartialEq, Eq, Clone)]
pub enum Mode {
    Browser,
    Queue,
    Playlist,
    Settings,
}

fn draw_log(area: Rect, buf: &mut Buffer) -> Rect {
    if let Some(msg) = log::last_message() {
        let area = layout!(
            area,
            Direction::Vertical,
            Constraint::Min(2),
            Constraint::Length(3)
        );

        //TODO: Rework this.
        let text = text![msg];
        let temp = &[text];
        let lines = lines(
            temp,
            Some(block(None, 0, Borders::ALL, BorderType::Rounded, style())),
            None,
        );
        lines.draw(area[1], buf);

        // f.render_widget(
        //     Paragraph::new(msg).alignment(Alignment::Left).block(
        //         Block::default()
        //             .borders(Borders::ALL)
        //             .border_type(BorderType::Rounded),
        //     ),
        //     area[1],
        // );

        area[0]
    } else {
        area
    }
}

// const SEARCH_MARGIN: Margin = Margin {
//     vertical: 6,
//     horizontal: 8,
// };

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let mut persist = gonk_core::settings::Settings::new();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut scan_timer = Instant::now();
    let mut scan_handle = None;

    if !args.is_empty() {
        match args[0].as_str() {
            "add" => {
                if args.len() == 1 {
                    return Ok(println!("Usage: gonk add <path>"));
                }
                let path = args[1..].join(" ");
                let Ok(path) = fs::canonicalize(path) else {
                    return Ok(println!("Invalid path."));
                };
                let Some(path) = path.to_str() else {
                    return Ok(println!("Invalid path."));
                };
                if Path::new(&path).exists() {
                    persist.music_folder = path.to_string();
                    scan_handle = Some(db::create(path));
                    scan_timer = Instant::now();
                } else {
                    return Ok(println!("Invalid path."));
                }
            }
            "reset" => {
                return match gonk_core::db::reset() {
                    Ok(_) => Ok(println!("Database reset!")),
                    Err(e) => Ok(println!("Failed to reset database! {e}")),
                };
            }
            "help" | "--help" => {
                println!("Usage");
                println!("   gonk [<command> <args>]");
                println!();
                println!("Options");
                println!("   add   <path>  Add music to the library");
                println!("   reset         Reset the database");
                return Ok(());
            }
            _ if !args.is_empty() => return Ok(println!("Invalid command.")),
            _ => (),
        }
    }

    let (output_handle, input_handle) = handles();
    let (width, height) = info(output_handle).window_size;

    let mut viewport = Rect::new(0, 0, width, height);
    let mut buffers: [Buffer; 2] = [Buffer::empty(viewport), Buffer::empty(viewport)];
    let mut current = 0;

    //Prevents panic messages from being hidden.
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // let mut stdout = stdout();
        // disable_raw_mode();
        // disable_mouse_caputure();
        // leave_alternate_screen(&mut stdout);
        // show_cursor(&mut stdout);
        // stdout.flush().unwrap();

        orig_hook(panic_info);
        std::process::exit(1);
    }));

    let mut stdout = stdout();
    enter_alternate_screen(&mut stdout);
    hide_cursor(&mut stdout);
    clear(&mut stdout);

    let index = if persist.queue.is_empty() {
        None
    } else {
        Some(persist.index as usize)
    };

    let songs = Index::new(persist.queue.clone(), index);
    let ui_index = index.unwrap_or(0);
    let mut player = Player::new(
        &persist.output_device,
        persist.volume,
        songs,
        persist.elapsed,
    );

    let mut db = Database::new();
    let mut queue = Queue::new(ui_index, addr_of_mut!(player));
    let mut browser = Browser::new(&db);
    let mut playlist = Playlist::new()?;
    let mut settings = Settings::new(&persist.output_device);
    let mut search = Search::new();
    let mut mode = Mode::Browser;
    let mut last_tick = Instant::now();
    let mut dots: usize = 1;

    //If there are songs in the queue and the database isn't scanning, display the queue.
    if !player.songs.is_empty() && scan_handle.is_none() {
        mode = Mode::Queue;
    }

    let mut searching = false;
    let mut help = false;

    loop {
        if let Some(handle) = &scan_handle {
            if handle.is_finished() {
                let handle = scan_handle.take().unwrap();
                let result = handle.join().unwrap();

                db = Database::new();

                log::clear();

                match result {
                    db::ScanResult::Completed => {
                        log!(
                            "Finished adding {} files in {:.2} seconds.",
                            db.len,
                            scan_timer.elapsed().as_secs_f32()
                        );
                    }
                    db::ScanResult::CompletedWithErrors(errors) => {
                        let dir = "See %appdata%/gonk/gonk.log for details.";
                        let len = errors.len();
                        let s = if len == 1 { "" } else { "s" };

                        log!(
                            "Added {} files with {len} error{s}. {dir}",
                            db.len.saturating_sub(len)
                        );

                        let path = gonk_path().join("gonk.log");
                        let errors = errors.join("\n");
                        fs::write(path, errors)?;
                    }
                    db::ScanResult::FileInUse => {
                        log!("Could not update database, file in use.")
                    }
                }

                browser::refresh(&mut browser, &db);
                search.results = Index::new(db.search(&search.query), None);

                //No need to reset scan_timer since it's reset with new scans.
                scan_handle = None;
            }
        }

        if last_tick.elapsed() >= Duration::from_millis(150) {
            if scan_handle.is_some() {
                if dots < 3 {
                    dots += 1;
                } else {
                    dots = 1;
                }
                log!("Scanning for files{}", ".".repeat(dots));
            }

            //Update the time elapsed.
            persist.index = player.songs.index().unwrap_or(0) as u16;
            persist.elapsed = player.elapsed().as_secs_f32();
            persist.queue = player.songs.to_vec();
            persist.save()?;

            //Update the list of output devices
            //TODO: I don't like how this is done.
            settings.devices = Wasapi::devices();
            let mut index = settings.index.unwrap_or(0);
            if index >= settings.devices.len() {
                index = settings.devices.len().saturating_sub(1);
                settings.index = Some(index);
            }

            last_tick = Instant::now();
        }

        //Update the UI index.
        queue.len = player.songs.len();

        //TODO: This is very slow and causes scrolling to feel laggy.
        player.update();

        let input_playlist = playlist.mode == PlaylistMode::Popup && mode == Mode::Playlist;

        //Widgets
        {
            let buf = &mut buffers[current];
            let area = draw_log(viewport, buf);

            // mode = Mode::Browser;
            mode = Mode::Settings;
            match mode {
                Mode::Browser => browser::draw(&mut browser, area, buf, None),
                Mode::Settings => settings::draw(&mut settings, area, buf),
                _ => (),
            }

            // match mode {
            //     Mode::Browser => browser::draw(&mut browser, f, top, None),
            //     Mode::Queue => queue::draw(&mut queue, f, top, None),
            //     Mode::Playlist => playlist::draw(&mut playlist, f, top, None),
            //     Mode::Settings => settings::draw(&mut settings, f, top),
            // }

            // if help {
            //     let area = top.inner(&SEARCH_MARGIN);
            //     f.render_widget(tui::widgets::Clear, area);
            //     let widths = [Constraint::Percentage(50), Constraint::Percentage(50)];

            //     //TODO: This is hard to read because the gap between command and key is large.
            //     let table = Table::new(&HELP)
            //         .header(
            //             Row::new(["Command", "Key"])
            //                 .style(
            //                     Style::default()
            //                         .fg(Color::White)
            //                         .add_modifier(Modifier::BOLD),
            //                 )
            //                 .bottom_margin(1),
            //         )
            //         .block(
            //             Block::default()
            //                 .title("Help:")
            //                 .borders(Borders::ALL)
            //                 .border_type(BorderType::Rounded),
            //         )
            //         .widths(&widths);

            //     f.render_widget(table, area);
            // } else if searching {
            //     search::draw(&mut search, f, top, None, &db);
            // }
        }

        //Rendering
        {
            //Calculate difference and draw to the terminal.
            let previous_buffer = &buffers[1 - current];
            let current_buffer = &buffers[current];
            let diff = previous_buffer.diff(current_buffer);
            buffer::draw(&mut stdout, diff);

            //Swap buffers
            buffers[1 - current].reset();
            current = 1 - current;

            //Update the viewport area.
            //TODO: I think there is a resize event that might be better.
            let (width, height) = info(output_handle).window_size;
            viewport = Rect::new(0, 0, width, height);

            //Resize
            if buffers[current].area != viewport {
                buffers[current].resize(viewport);
                buffers[1 - current].resize(viewport);

                // Reset the back buffer to make sure the next update will redraw everything.
                buffers[1 - current].reset();
                clear(&mut stdout);
            }
        }
    }

    //     if !event::poll(Duration::from_millis(2))? {
    //         continue;
    //     }

    //     match event::read()? {
    //         Event::Mouse(mouse_event) if !help => match mouse_event.kind {
    //             MouseEventKind::ScrollUp if searching => search::up(&mut search),
    //             MouseEventKind::ScrollDown if searching => search::down(&mut search),
    //             MouseEventKind::ScrollUp => match mode {
    //                 Mode::Browser => browser::up(&mut browser, &db),
    //                 Mode::Queue => queue::up(&mut queue),
    //                 Mode::Playlist => playlist::up(&mut playlist),
    //                 Mode::Settings => settings::up(&mut settings),
    //             },
    //             MouseEventKind::ScrollDown => match mode {
    //                 Mode::Browser => browser::down(&mut browser, &db),
    //                 Mode::Queue => queue::down(&mut queue),
    //                 Mode::Playlist => playlist::down(&mut playlist),
    //                 Mode::Settings => settings::down(&mut settings),
    //             },
    //             MouseEventKind::Down(_) => {
    //                 //FIXME: Copy-paste drawing. Need to swap to custom tui library.
    //                 //tui-rs has private fields which makes the api to inflexable for my use.
    //                 terminal.draw(|f| {
    //                     let top = draw_log(f);
    //                     let event = if searching { None } else { Some(mouse_event) };

    //                     //TODO: Remove mouse_event from draw.
    //                     match mode {
    //                         Mode::Browser => browser::draw(&mut browser, f, top, event),
    //                         Mode::Queue => queue::draw(&mut queue, f, top, event),
    //                         Mode::Playlist => playlist::draw(&mut playlist, f, top, event),
    //                         Mode::Settings => settings::draw(&mut settings, f, top),
    //                     }

    //                     if searching {
    //                         search::draw(&mut search, f, top, Some(mouse_event), &db);
    //                     }
    //                 })?;
    //             }
    //             _ => (),
    //         },
    //         Event::Key(event) => {
    //             let shift = event.modifiers == KeyModifiers::SHIFT;
    //             let control = event.modifiers == KeyModifiers::CONTROL;

    //             match event.code {
    //                 KeyCode::Char('c') if control => break,
    //                 _ if help => match event.code {
    //                     KeyCode::Char('?') | KeyCode::Char('/') | KeyCode::Esc => help = false,
    //                     _ => (),
    //                 },
    //                 _ if searching => {
    //                     match event.code {
    //                         KeyCode::Char('/') => match search.mode {
    //                             SearchMode::Search if search.query.is_empty() => {
    //                                 searching = false;
    //                             }
    //                             SearchMode::Search => {
    //                                 search.query.push('/');
    //                                 search.query_changed = true;
    //                             }
    //                             SearchMode::Select => {
    //                                 search.mode = SearchMode::Search;
    //                                 search.results.select(None);
    //                             }
    //                         },
    //                         KeyCode::Char(c) if search.mode == SearchMode::Search => {
    //                             //Handle ^W as control backspace.
    //                             if control && c == 'w' {
    //                                 search::on_backspace(&mut search, true);
    //                             } else {
    //                                 search.query.push(c);
    //                                 search.query_changed = true;
    //                             }
    //                         }
    //                         KeyCode::Up | KeyCode::Char('k') => search::up(&mut search),
    //                         KeyCode::Down | KeyCode::Char('j') => search::down(&mut search),
    //                         KeyCode::Backspace => match search.mode {
    //                             SearchMode::Search if !search.query.is_empty() => {
    //                                 if control {
    //                                     search.query.clear();
    //                                 } else {
    //                                     search.query.pop();
    //                                 }

    //                                 search.query_changed = true;
    //                             }
    //                             SearchMode::Search => (),
    //                             SearchMode::Select => {
    //                                 search.results.select(None);
    //                                 search.mode = SearchMode::Search;
    //                             }
    //                         },
    //                         KeyCode::Esc => match search.mode {
    //                             SearchMode::Search => {
    //                                 searching = false;
    //                                 search.query = String::new();
    //                                 search.query_changed = true;
    //                             }
    //                             SearchMode::Select => {
    //                                 searching = false;

    //                                 search.mode = SearchMode::Search;
    //                                 search.results.select(None);
    //                                 search.query = String::new();
    //                                 search.query_changed = true;
    //                             }
    //                         },
    //                         KeyCode::Enter if shift && searching => {
    //                             if let Some(songs) = search::on_enter(&mut search, &db) {
    //                                 playlist::add(
    //                                     &mut playlist,
    //                                     songs.iter().map(|song| song.clone().clone()).collect(),
    //                                 );
    //                                 mode = Mode::Playlist;
    //                             }
    //                         }
    //                         KeyCode::Enter => {
    //                             if let Some(songs) = search::on_enter(&mut search, &db) {
    //                                 //Swap to the queue so people can see what they added.
    //                                 mode = Mode::Queue;
    //                                 let songs: Vec<Song> =
    //                                     songs.iter().map(|song| song.clone().clone()).collect();
    //                                 player.add(songs);
    //                             }
    //                         }
    //                         _ => (),
    //                     }
    //                 }
    //                 KeyCode::Char('?') => help = true,
    //                 KeyCode::Char('/') => searching = true,
    //                 KeyCode::Char(c) if input_playlist => {
    //                     if control && c == 'w' {
    //                         playlist::on_backspace(&mut playlist, true);
    //                     } else {
    //                         playlist.changed = true;
    //                         playlist.search_query.push(c);
    //                     }
    //                 }
    //                 KeyCode::Char(' ') => player.toggle_playback(),
    //                 KeyCode::Char('C') => {
    //                     player.clear_except_playing();
    //                     queue.ui.select(Some(0));
    //                 }
    //                 KeyCode::Char('c') => {
    //                     player.clear();
    //                     queue.ui.select(Some(0));
    //                 }
    //                 KeyCode::Char('x') => match mode {
    //                     Mode::Queue => {
    //                         if let Some(i) = queue.ui.index() {
    //                             player.delete_index(i);

    //                             //Sync the UI index.
    //                             let len = player.songs.len().saturating_sub(1);
    //                             if i > len {
    //                                 queue.ui.select(Some(len));
    //                             }
    //                         }
    //                     }
    //                     Mode::Playlist => {
    //                         playlist::delete(&mut playlist, false);
    //                     }
    //                     _ => (),
    //                 },
    //                 KeyCode::Char('X') if mode == Mode::Playlist => {
    //                     playlist::delete(&mut playlist, false)
    //                 }
    //                 KeyCode::Char('u') if mode == Mode::Browser || mode == Mode::Playlist => {
    //                     if scan_handle.is_none() {
    //                         if persist.music_folder.is_empty() {
    //                             gonk_core::log!(
    //                                 "Nothing to scan! Add a folder with 'gonk add /path/'"
    //                             );
    //                         } else {
    //                             scan_handle = Some(db::create(persist.music_folder.clone()));
    //                             scan_timer = Instant::now();
    //                             playlist.lists = Index::from(gonk_core::playlist::playlists());
    //                         }
    //                     }
    //                 }
    //                 KeyCode::Char('z') => player.mute(),
    //                 KeyCode::Char('q') => player.seek_backward(),
    //                 KeyCode::Char('e') => player.seek_foward(),
    //                 KeyCode::Char('a') => player.prev(),
    //                 KeyCode::Char('d') => player.next(),
    //                 KeyCode::Char('w') => {
    //                     player.volume_up();
    //                     persist.volume = player.volume();
    //                 }
    //                 KeyCode::Char('s') => {
    //                     player.volume_down();
    //                     persist.volume = player.volume();
    //                 }
    //                 KeyCode::Esc if mode == Mode::Playlist => {
    //                     if playlist.delete {
    //                         playlist.yes = true;
    //                         playlist.delete = false;
    //                     } else if let playlist::Mode::Popup = playlist.mode {
    //                         playlist.mode = playlist::Mode::Playlist;
    //                         playlist.search_query = String::new();
    //                         playlist.changed = true;
    //                     }
    //                 }
    //                 KeyCode::Enter if shift => match mode {
    //                     Mode::Browser => {
    //                         playlist::add(&mut playlist, browser::get_selected(&browser, &db));
    //                         mode = Mode::Playlist
    //                     }
    //                     Mode::Queue => {
    //                         if let Some(index) = queue.ui.index() {
    //                             if let Some(song) = player.songs.get(index) {
    //                                 playlist::add(&mut playlist, vec![song.clone()]);
    //                                 mode = Mode::Playlist;
    //                             }
    //                         }
    //                     }

    //                     _ => (),
    //                 },
    //                 KeyCode::Enter => match mode {
    //                     Mode::Browser => {
    //                         player.add(browser::get_selected(&browser, &db));
    //                     }
    //                     Mode::Queue => {
    //                         if let Some(i) = queue.ui.index() {
    //                             player.play_index(i);
    //                         }
    //                     }
    //                     Mode::Settings => {
    //                         if let Some(device) = settings::selected(&mut settings) {
    //                             let device = device.to_string();
    //                             player.set_output_device(&device);
    //                             settings.current_device = device.clone();
    //                             persist.output_device = device.clone();
    //                         }
    //                     }
    //                     Mode::Playlist => playlist::on_enter(&mut playlist, &mut player),
    //                 },
    //                 KeyCode::Backspace => {
    //                     if let Mode::Playlist = mode {
    //                         playlist::on_backspace(&mut playlist, control)
    //                     }
    //                 }
    //                 KeyCode::Char('1') => mode = Mode::Queue,
    //                 KeyCode::Char('2') => mode = Mode::Browser,
    //                 KeyCode::Char('3') => mode = Mode::Playlist,
    //                 KeyCode::Char('4') => mode = Mode::Settings,
    //                 KeyCode::F(1) => queue::constraint(&mut queue, 0, shift),
    //                 KeyCode::F(2) => queue::constraint(&mut queue, 1, shift),
    //                 KeyCode::F(3) => queue::constraint(&mut queue, 2, shift),
    //                 KeyCode::Up | KeyCode::Char('k') => match mode {
    //                     Mode::Browser => browser::up(&mut browser, &db),
    //                     Mode::Queue => queue::up(&mut queue),
    //                     Mode::Playlist => playlist::up(&mut playlist),
    //                     Mode::Settings => settings::up(&mut settings),
    //                 },
    //                 KeyCode::Down | KeyCode::Char('j') => match mode {
    //                     Mode::Browser => browser::down(&mut browser, &db),
    //                     Mode::Queue => queue::down(&mut queue),
    //                     Mode::Playlist => playlist::down(&mut playlist),
    //                     Mode::Settings => settings::down(&mut settings),
    //                 },
    //                 KeyCode::Left | KeyCode::Char('h') => match mode {
    //                     Mode::Browser => browser::left(&mut browser),
    //                     Mode::Queue => (),
    //                     Mode::Playlist => playlist::left(&mut playlist),
    //                     Mode::Settings => (),
    //                 },
    //                 KeyCode::Right | KeyCode::Char('l') => match mode {
    //                     Mode::Browser => browser::right(&mut browser),
    //                     Mode::Queue => (),
    //                     Mode::Playlist => playlist::right(&mut playlist),
    //                     Mode::Settings => (),
    //                 },
    //                 _ => (),
    //             }
    //         }
    //         _ => (),
    //     }
    // }

    persist.queue = player.songs.to_vec();
    persist.index = player.songs.index().unwrap_or(0) as u16;
    persist.elapsed = player.elapsed().as_secs_f32();
    persist.save()?;

    // disable_raw_mode()?;
    // execute!(
    //     terminal.backend_mut(),
    //     LeaveAlternateScreen,
    //     DisableMouseCapture
    // )?;
    gonk_core::profiler::print();

    Ok(())
}
