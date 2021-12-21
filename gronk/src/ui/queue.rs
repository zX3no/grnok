use crate::app::App;
use crate::ui::{ALBUM, ARTIST, TITLE, TRACK};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState};
use tui::Frame;

//TODO: store the duration in the database
//abstract selection color into it's own widget
pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(20),
            Constraint::Length(2),
        ])
        .split(f.size());

    draw_header(f, app, chunks[0]);
    draw_songs(f, app, chunks[1]);
    draw_seeker(f, app, chunks[2]);
}

pub fn draw_header<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let playback = if app.queue.is_empty() {
        "[stopped]"
    } else if app.queue.is_playing() {
        "[playing]"
    } else {
        "[paused]"
    };

    let time = if let Some(duration) = app.queue.duration() {
        let elapsed = app.queue.elapsed().as_secs_f64();
        let duration = duration.as_secs_f64();

        let mins = elapsed / 60.0;
        let rem = elapsed % 60.0;
        let e = format!(
            "{:0width$}:{:0width$}",
            mins.trunc() as usize,
            rem.trunc() as usize,
            width = 2,
        );

        let mins = duration / 60.0;
        let rem = duration % 60.0;
        let d = format!(
            "{:0width$}:{:0width$}",
            mins.trunc() as usize,
            rem.trunc() as usize,
            width = 2,
        );

        format!("{}/{}", e, d)
    } else {
        String::from("0:00/0:00")
    };

    let left = vec![Spans::from(time), Spans::from(playback)];

    let spacer: String = {
        let width = chunk.width;
        (0..width - 4).map(|_| "-").collect()
    };

    let center = if let Some(song) = app.queue.get_playing() {
        //I wish that paragraphs had clipping
        //I think constaints do
        //I could render the -| |- on a seperate layer
        //outside of the constraint which might work better?
        let mut name = song.name.clone();
        while (name.len() + song.artist.len() + "-| - |-".len()) > (chunk.width - 40) as usize {
            name.pop();
        }
        let name = name.trim_end().to_string();

        vec![
            Spans::from(vec![
                Span::raw("─| "),
                Span::styled(&song.artist, Style::default().fg(ARTIST)),
                Span::raw(" - "),
                Span::styled(name, Style::default().fg(TITLE)),
                Span::raw(" |─"),
            ]),
            Spans::from(Span::styled(&song.album, Style::default().fg(ALBUM))),
            Spans::default(),
            Spans::from(spacer),
        ]
    } else {
        vec![
            Spans::default(),
            Spans::default(),
            Spans::default(),
            Spans::from(spacer),
        ]
    };

    let volume = app.queue.get_volume_percent();

    let right = Spans::from(format!("Vol: {}%", volume));

    let left = Paragraph::new(left).alignment(Alignment::Left);
    let center = Paragraph::new(center).alignment(Alignment::Center);
    let right = Paragraph::new(right).alignment(Alignment::Right);

    let b = Block::default()
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .border_type(BorderType::Rounded);

    f.render_widget(b, chunk);

    let c = Rect {
        x: chunk.x + 2,
        y: chunk.y,
        width: chunk.width - 2,
        height: chunk.height,
    };

    f.render_widget(left, c);

    let c = Rect {
        x: chunk.x,
        y: chunk.y,
        width: chunk.width,
        height: chunk.height,
    };

    f.render_widget(center, c);

    let c = Rect {
        x: chunk.x,
        y: chunk.y,
        width: chunk.width - 2,
        height: chunk.height,
    };

    f.render_widget(right, c);
}

pub fn draw_songs<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let (songs, now_playing, ui_index) = (
        &app.queue.list.songs,
        &app.queue.list.now_playing,
        &app.queue.ui_index.index,
    );

    let mut items: Vec<Row> = songs
        .iter()
        .map(|song| {
            Row::new(vec![
                Cell::from(""),
                Cell::from(song.number.to_string()).style(Style::default().fg(TRACK)),
                Cell::from(song.name.to_owned()).style(Style::default().fg(TITLE)),
                Cell::from(song.album.to_owned()).style(Style::default().fg(ALBUM)),
                Cell::from(song.artist.to_owned()).style(Style::default().fg(ARTIST)),
            ])
        })
        .collect();

    if let Some(playing_index) = now_playing {
        if let Some(song) = songs.get(*playing_index) {
            if let Some(ui_index) = ui_index {
                //currently playing song
                let row = if ui_index == playing_index {
                    Row::new(vec![
                        Cell::from(">>").style(
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::DIM | Modifier::BOLD),
                        ),
                        Cell::from(song.number.to_string())
                            .style(Style::default().bg(TRACK).fg(Color::Black)),
                        Cell::from(song.name.to_owned())
                            .style(Style::default().bg(TITLE).fg(Color::Black)),
                        Cell::from(song.album.to_owned())
                            .style(Style::default().bg(ALBUM).fg(Color::Black)),
                        Cell::from(song.artist.to_owned())
                            .style(Style::default().bg(ARTIST).fg(Color::Black)),
                    ])
                } else {
                    Row::new(vec![
                        Cell::from(">>").style(
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::DIM | Modifier::BOLD),
                        ),
                        Cell::from(song.number.to_string()).style(Style::default().fg(TRACK)),
                        Cell::from(song.name.to_owned()).style(Style::default().fg(TITLE)),
                        Cell::from(song.album.to_owned()).style(Style::default().fg(ALBUM)),
                        Cell::from(song.artist.to_owned()).style(Style::default().fg(ARTIST)),
                    ])
                };
                items.remove(*playing_index);
                items.insert(*playing_index, row);

                //current selection
                if ui_index != playing_index {
                    let song = songs.get(*ui_index).unwrap();
                    let row = Row::new(vec![
                        Cell::from(""),
                        Cell::from(song.number.to_string()).style(Style::default().bg(TRACK)),
                        Cell::from(song.name.to_owned()).style(Style::default().bg(TITLE)),
                        Cell::from(song.album.to_owned()).style(Style::default().bg(ALBUM)),
                        Cell::from(song.artist.to_owned()).style(Style::default().bg(ARTIST)),
                    ])
                    .style(Style::default().fg(Color::Black));
                    items.remove(*ui_index);
                    items.insert(*ui_index, row);
                }
            }
        }
    }

    let con = [
        Constraint::Length(2),
        Constraint::Percentage(app.constraint[0]),
        Constraint::Percentage(app.constraint[1]),
        Constraint::Percentage(app.constraint[2]),
        Constraint::Percentage(app.constraint[3]),
    ];

    let t = Table::new(items)
        .header(
            Row::new(vec!["", "Track", "Title", "Album", "Artist"])
                .style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded),
        )
        .widths(&con);

    //this is so that scrolling works
    let mut state = TableState::default();
    state.select(*ui_index);

    f.render_stateful_widget(t, chunk, &mut state);
}
pub fn draw_seeker<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: Rect) {
    if app.queue.is_empty() {
        return f.render_widget(
            Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded),
            chunk,
        );
    }

    if let Some((column, row)) = app.clicked_pos {
        let size = f.size();
        //row = 28
        //height = 30
        if size.height - 3 == row || size.height - 2 == row || size.height - 1 == row {
            let ratio = (column - 4) as f64 / size.width as f64;
            let duration = app.queue.duration().unwrap().as_secs_f64();

            let new_time = duration * ratio;
            app.queue.seek_to(new_time);
            app.queue.play();
        }
        app.clicked_pos = None;
    }

    let area = f.size();
    let width = area.width;
    let percent = app.seeker;
    let pos = (width as f64 * percent).ceil() as usize;

    let mut string = String::new();
    for i in 0..(width - 6) {
        if (i as usize) < pos {
            string.push('=');
        } else {
            string.push('-');
        }
    }

    //place the seeker location
    if pos < string.len() - 1 {
        string.remove(pos);
        string.insert(pos, '>');
    } else {
        string.pop();
        string.push('>');
    }

    let p = Paragraph::new(string).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_type(BorderType::Rounded),
    );

    f.render_widget(p, chunk)
}
