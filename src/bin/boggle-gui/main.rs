use boggle::file::stdio::*;
use boggle::BChar::*;
use boggle::*;
use glib::clone;
use gtk::glib;
use gtk::pango;
use gtk::prelude::*;
use rand::prelude::*;
use rand::{rngs::ThreadRng, thread_rng};
use std::cell::{Ref, RefCell, RefMut};
use std::f64::consts::{FRAC_PI_2, PI};
use std::rc::Rc;
use std::time;

const GAME_DUR: time::Duration = time::Duration::from_secs(90);
const UPDATE_FREQ: time::Duration = time::Duration::from_millis(5);

struct State {
    rng: ThreadRng,
    dict: Dict,
    board: Option<Board>,
}

impl State {
    fn gen_board(&mut self, dice: &Dice) {
        self.board = Some(roll(dice, &mut self.rng));
    }
}

fn main() {
    let app = gtk::Application::new(Some("com.github.shrimpy48.boggle"), Default::default());
    app.connect_activate(build);
    app.run();
}

fn build(app: &gtk::Application) {
    let ui_src = include_str!("structure.ui");
    let builder = gtk::Builder::from_string(ui_src);

    let window: gtk::ApplicationWindow = builder.object("window").expect("couldn't get window");
    let stack: gtk::Stack = builder.object("pagestack").expect("couldn't get pagestack");
    let play_button: gtk::Button = builder
        .object("playbutton")
        .expect("couldn't get playbutton");
    let play_again_button: gtk::Button = builder
        .object("playagainbutton")
        .expect("couldn't get playagainbutton");
    let menu_button: gtk::Button = builder
        .object("menubutton")
        .expect("couldn't get menubutton");
    let board_view: gtk::DrawingArea = builder.object("board").expect("couldn't get board");
    let entered_view: gtk::ListView = builder
        .object("enteredview")
        .expect("couldn't get enteredview");
    let entry: gtk::Entry = builder.object("entry").expect("couldn't get entry");
    let correct_view: gtk::ListView = builder
        .object("correctview")
        .expect("couldn't get correctview");
    let not_word_view: gtk::ListView = builder
        .object("notwordview")
        .expect("couldn't get notwordview");
    let other_view: gtk::ListView = builder.object("otherview").expect("couldn't get otherview");
    let total_score_label: gtk::Label = builder
        .object("totalscorelabel")
        .expect("couldn't get totalscorelabel");
    let timebar: gtk::ProgressBar = builder.object("timebar").expect("couldn't get timebar");

    let ctx = board_view.pango_context();
    let layout = pango::Layout::new(&ctx);
    let desc = pango::FontDescription::from_string("sans bold 20");
    layout.set_font_description(Some(&desc));
    let mut size = 0;
    for bc in [
        A, B, C, D, E, F, G, H, I, J, K, L, O, P, Qu, R, S, T, U, V, X, Y,
    ] {
        layout.set_text(&bc.to_string());
        let (w, h) = layout.size();
        size = size.max(w).max(h);
    }
    let attr_list = pango::AttrList::new();
    attr_list.insert(pango::Attribute::new_underline(pango::Underline::Low));
    layout.set_attributes(Some(&attr_list));
    for bc in [M, N, W, Z] {
        layout.set_text(&bc.to_string());
        let (w, h) = layout.size();
        size = size.max(w).max(h);
    }
    let size = (pango::units_to_double(size) * 4.).round() as i32;
    board_view.set_content_width(size);
    board_view.set_content_height(size);

    let dice = read_dice("dice.txt").unwrap();
    let state = Rc::new(RefCell::new(State {
        rng: thread_rng(),
        dict: read_dict("dictionaries/custom.txt").unwrap(),
        board: None,
    }));

    let score_sorter = new_score_sorter();

    let entered_model = gtk::StringList::new(&[]);
    let correct_model = gtk::StringList::new(&[]);
    let not_word_model = gtk::StringList::new(&[]);
    let other_model = gtk::StringList::new(&[]);

    let plain_factory = new_plain_factory();
    let correct_factory = new_correct_factory(
        clone!(@weak total_score_label, @weak correct_model, @weak not_word_model, @weak other_model, @strong state => move |string| {
            state.borrow_mut().dict.remove(&string.parse::<BString>().unwrap());
            let mut total_score: u8 = total_score_label.text().parse().expect("score label text not integer");
            let mut i = 0;
            let mut n = correct_model.n_items();
            while i < n {
                if correct_model.string(i).as_ref() == Some(string) {
                    correct_model.remove(i);
                    not_word_model.append(string);
                    total_score -= score(string);
                    n -= 1;
                } else {
                    i += 1;
                }
            }
            let mut i = 0;
            let mut n = other_model.n_items();
            while i < n {
                if other_model.string(i).as_ref() == Some(string) {
                    other_model.remove(i);
                    n -= 1;
                } else {
                    i += 1;
                }
            }
            total_score_label.set_text(&total_score.to_string());
        }),
    );
    let incorrect_factory = new_incorrect_factory(
        clone!(@weak total_score_label, @weak correct_model, @weak not_word_model, @strong state => move |string| {
            state.borrow_mut().dict.insert(&string.parse::<BString>().unwrap());
            let mut total_score: u8 = total_score_label.text().parse().expect("score label text not integer");
            let mut i = 0;
            let mut n = not_word_model.n_items();
            while i < n {
                if not_word_model.string(i).as_ref() == Some(string) {
                    not_word_model.remove(i);
                    correct_model.append(string);
                    total_score += score(string);
                    n -= 1;
                } else {
                    i += 1;
                }
            }
            total_score_label.set_text(&total_score.to_string());
        }),
    );

    let selection_model = gtk::NoSelection::new(Some(&entered_model));
    entered_view.set_model(Some(&selection_model));
    entered_view.set_factory(Some(&plain_factory));

    let selection_model = gtk::NoSelection::new(Some(&gtk::SortListModel::new(
        Some(&correct_model),
        Some(&score_sorter),
    )));
    correct_view.set_model(Some(&selection_model));
    correct_view.set_factory(Some(&correct_factory));

    let selection_model = gtk::NoSelection::new(Some(&not_word_model));
    not_word_view.set_model(Some(&selection_model));
    not_word_view.set_factory(Some(&incorrect_factory));

    let selection_model = gtk::NoSelection::new(Some(&gtk::SortListModel::new(
        Some(&other_model),
        Some(&score_sorter),
    )));
    other_view.set_model(Some(&selection_model));
    other_view.set_factory(Some(&correct_factory));

    window.set_application(Some(app));

    play_button.connect_clicked(
        clone!(@weak window, @weak stack, @weak total_score_label, @weak board_view, @weak entry, @weak entered_model, @weak correct_model, @weak not_word_model, @weak other_model, @strong timebar, @strong state => move |_| {
            start_game(&stack, &board_view, &entry, &entered_model, &dice, state.borrow_mut());
            let start_time = time::Instant::now();
            glib::source::timeout_add_local(UPDATE_FREQ, clone!(@strong timebar => move || {
                let current_time = time::Instant::now();
                let fraction = ((current_time - start_time).as_secs_f64() / GAME_DUR.as_secs_f64()).clamp(0., 1.);
                timebar.set_fraction(fraction);
                Continue(fraction < 1.)
            }));
            glib::source::timeout_add_local_once(GAME_DUR, clone!(@weak window, @weak stack, @weak total_score_label, @weak entered_model, @weak correct_model, @weak not_word_model, @weak other_model, @strong state => move || {
                window.set_focus(Option::<&gtk::Widget>::None);
                show_results(&stack, &total_score_label, &entered_model, &correct_model, &not_word_model, &other_model, state.borrow());
            }));
        }),
    );
    play_again_button.connect_clicked(
        clone!(@weak window, @weak stack, @weak total_score_label, @weak board_view, @weak entry, @weak entered_model, @weak correct_model, @weak not_word_model, @weak other_model, @strong timebar, @strong state => move |_| {
            start_game(&stack, &board_view, &entry, &entered_model, &dice, state.borrow_mut());
            let start_time = time::Instant::now();
            glib::source::timeout_add_local(UPDATE_FREQ, clone!(@strong timebar => move || {
                let current_time = time::Instant::now();
                let fraction = ((current_time - start_time).as_secs_f64() / GAME_DUR.as_secs_f64()).clamp(0., 1.);
                timebar.set_fraction(fraction);
                Continue(fraction < 1.)
            }));
            glib::source::timeout_add_local_once(GAME_DUR, clone!(@weak window, @weak stack, @weak total_score_label, @weak entered_model, @weak correct_model, @weak not_word_model, @weak other_model, @strong state => move || {
                window.set_focus(Option::<&gtk::Widget>::None);
                show_results(&stack, &total_score_label, &entered_model, &correct_model, &not_word_model, &other_model, state.borrow());
            }));
        }),
    );
    menu_button.connect_clicked(clone!(@weak stack, @weak board_view => move |_| {
        board_view.hide();
        stack.set_visible_child_name("menu");
    }));
    entry.connect_activate(clone!(@weak entered_view, @strong state => move |entry| {
        let word = entry.text();
        entered_model.append(&word);
        entry.set_text("");
    }));

    window.connect_close_request(clone!(@strong state => move |_| {
        if let Err(e) = write_dict("dictionaries/custom.txt", &state.borrow().dict) {
            eprintln!("Could not write dict: {}", e);
        }
        gtk::Inhibit(false)
    }));

    board_view.hide();
    window.present();
}

fn new_score_sorter() -> gtk::NumericSorter {
    let score_sorter = gtk::NumericSorter::new(Some(&gtk::ClosureExpression::new(
        |val| {
            score(
                &val[0]
                    .get::<gtk::StringObject>()
                    .expect("value not a `StringObject`")
                    .string(),
            )
        },
        &[],
    )));
    score_sorter.set_sort_order(gtk::SortType::Descending);
    score_sorter
}

fn new_plain_factory() -> gtk::SignalListItemFactory {
    let plain_factory = gtk::SignalListItemFactory::new();
    plain_factory.connect_setup(move |_, list_item| {
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
    });
    plain_factory.connect_bind(move |_, list_item| {
        let str_obj = list_item
            .item()
            .expect("no item")
            .downcast::<gtk::StringObject>()
            .expect("item not a `StringObject`");
        let string = str_obj.string();

        let label = list_item
            .child()
            .expect("no child")
            .downcast::<gtk::Label>()
            .expect("child not a `Label`");

        label.set_label(&string);
    });
    plain_factory
}

fn new_correct_factory<F: Fn(&glib::GString) + Clone + 'static>(
    button_callback: F,
) -> gtk::SignalListItemFactory {
    let correct_factory = gtk::SignalListItemFactory::new();
    correct_factory.connect_setup(move |_, list_item| {
        let rowbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let word_label = gtk::Label::new(None);
        rowbox.append(&word_label);
        let score_label = gtk::Label::new(None);
        rowbox.append(&score_label);
        let challenge_button = gtk::Button::with_label("Disallow");
        challenge_button.set_halign(gtk::Align::End);
        challenge_button.set_hexpand(true);
        rowbox.append(&challenge_button);
        list_item.set_child(Some(&rowbox));
    });
    correct_factory.connect_bind(move |_, list_item| {
        let str_obj = list_item
            .item()
            .expect("no item")
            .downcast::<gtk::StringObject>()
            .expect("item not a `StringObject`");
        let string = str_obj.string();

        let rowbox = list_item
            .child()
            .expect("no child")
            .downcast::<gtk::Box>()
            .expect("child not a `Box`");
        let word_label = rowbox
            .first_child()
            .expect("rowbox has no children")
            .downcast::<gtk::Label>()
            .expect("rowbox[0] not a `Label`");
        let score_label = word_label
            .next_sibling()
            .expect("rowbox has only 1 child")
            .downcast::<gtk::Label>()
            .expect("rowbox[1] not a `Label`");
        let challenge_button = rowbox
            .last_child()
            .expect("rowbox has no children")
            .downcast::<gtk::Button>()
            .expect("rowbox[-1] not a `Button`");

        word_label.set_label(&string);
        score_label.set_label(&score(&string).to_string());
        challenge_button
            .connect_clicked(clone!(@strong button_callback => move |_| button_callback(&string)));
    });
    correct_factory
}

fn new_incorrect_factory<F: Fn(&glib::GString) + Clone + 'static>(
    button_callback: F,
) -> gtk::SignalListItemFactory {
    let incorrect_factory = gtk::SignalListItemFactory::new();
    incorrect_factory.connect_setup(move |_, list_item| {
        let rowbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let word_label = gtk::Label::new(None);
        rowbox.append(&word_label);
        let challenge_button = gtk::Button::with_label("Allow");
        challenge_button.set_halign(gtk::Align::End);
        challenge_button.set_hexpand(true);
        rowbox.append(&challenge_button);
        list_item.set_child(Some(&rowbox));
    });
    incorrect_factory.connect_bind(move |_, list_item| {
        let str_obj = list_item
            .item()
            .expect("no item")
            .downcast::<gtk::StringObject>()
            .expect("item not a `StringObject`");
        let string = str_obj.string();

        let rowbox = list_item
            .child()
            .expect("no child")
            .downcast::<gtk::Box>()
            .expect("child not a `Box`");
        let word_label = rowbox
            .first_child()
            .expect("rowbox has no children")
            .downcast::<gtk::Label>()
            .expect("rowbox[0] not a `Label`");
        let challenge_button = rowbox
            .last_child()
            .expect("rowbox has no children")
            .downcast::<gtk::Button>()
            .expect("rowbox[-1] not a `Button`");

        word_label.set_label(&string);
        challenge_button
            .connect_clicked(clone!(@strong button_callback => move |_| button_callback(&string)));
    });
    incorrect_factory
}

fn start_game(
    stack: &gtk::Stack,
    board_view: &gtk::DrawingArea,
    entry: &gtk::Entry,
    entered_model: &gtk::StringList,
    dice: &Dice,
    mut state: RefMut<State>,
) {
    state.gen_board(dice);
    show_board(board_view, state);

    entered_model.splice(0, entered_model.n_items(), &[]);
    entry.set_text("");

    entry.grab_focus();
    board_view.show();
    stack.set_visible_child_name("game");
}

fn show_board(board_view: &gtk::DrawingArea, mut state: RefMut<State>) {
    let mut angles = [[0.; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            angles[r][c] = *[0., FRAC_PI_2, PI, PI + FRAC_PI_2]
                .choose(&mut state.rng)
                .unwrap();
        }
    }
    let board = state.board.as_ref().unwrap().clone();
    let desc = pango::FontDescription::from_string("sans bold 20");
    board_view.set_draw_func(move |_board_view, context, w, h| {
        let layout = pangocairo::create_layout(context).unwrap();
        layout.set_font_description(Some(&desc));
        for r in 0..4 {
            for c in 0..4 {
                layout.set_text(&board[r][c].to_string());
                if [M, N, W, Z].contains(&board[r][c]) {
                    let attr_list = pango::AttrList::new();
                    attr_list.insert(pango::Attribute::new_underline(pango::Underline::Low));
                    layout.set_attributes(Some(&attr_list));
                } else {
                    layout.set_attributes(Some(&pango::AttrList::new()));
                }
                context.save().unwrap();
                context.rotate(angles[r][c]);
                pangocairo::update_layout(context, &layout);
                let spacing = w.min(h) as f64 / 8.;
                let rs = spacing + r as f64 * spacing * 2.;
                let cs = spacing + c as f64 * spacing * 2.;
                let rr = rs * angles[r][c].cos() - cs * angles[r][c].sin();
                let cr = cs * angles[r][c].cos() + rs * angles[r][c].sin();
                let (w, h) = layout.size();
                context.move_to(
                    cr - pango::units_to_double(w) / 2.,
                    rr - pango::units_to_double(h) / 2.,
                );
                pangocairo::show_layout(context, &layout);
                context.restore().unwrap();
            }
        }
    });
}

fn show_results(
    stack: &gtk::Stack,
    total_score_label: &gtk::Label,
    entered_model: &gtk::StringList,
    correct_model: &gtk::StringList,
    not_word_model: &gtk::StringList,
    other_model: &gtk::StringList,
    state: Ref<State>,
) {
    let len = entered_model.n_items();
    let mut words: Vec<String> = Vec::with_capacity(len as usize);
    for i in 0..len {
        words.push(entered_model.string(i).unwrap().to_string());
    }
    let mut correct = Vec::new();
    let mut not_word = Vec::new();
    let mut total_score = 0;
    let mut present = state.board.as_ref().unwrap().words_trie(&state.dict);
    for (sword, bword) in words
        .iter()
        .flat_map(|w| w.parse::<BString>().map(|bw| (w, bw)))
    {
        if present.contains(&bword) {
            correct.push(sword);
            present.remove(&bword);
            total_score += score(sword);
        } else if sword.len() >= 3 && !state.dict.contains(&bword) {
            not_word.push(sword);
        }
    }
    let others = present.words();
    correct_model.splice(0, correct_model.n_items(), &[]);
    for word in correct {
        correct_model.append(&word);
    }
    not_word_model.splice(0, not_word_model.n_items(), &[]);
    for word in not_word {
        not_word_model.append(&word);
    }
    other_model.splice(0, other_model.n_items(), &[]);
    for word in others.into_iter().map(|w| w.to_string()) {
        other_model.append(&word);
    }
    total_score_label.set_text(&total_score.to_string());
    stack.set_visible_child_name("results");
}
