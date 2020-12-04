use pancurses::{Window, Input, ALL_MOUSE_EVENTS, BUTTON1_PRESSED};
use pancurses::{mousemask, mouseinterval, getmouse};
use pancurses::{initscr, cbreak, noecho, beep, napms, endwin};

const FPS: i32 = 20;
const MILBURN_LAUGHS: bool = true;

struct Target<'a> {
	y: i32, x: i32, going_down: bool, shape: Vec<&'a str>
}

impl Target<'_> {
	fn update(&mut self, max_y: i32) {
		if self.y == max_y - 1 || self.y == 0 {self.going_down = !self.going_down;}
		self.y += self.going_down as i32 * -2 + 1; // true => 1, false => -1
	}
	fn draw(&self, screen: &Window) {
		for row_index in 0 .. self.shape.len() {
			screen.mvprintw(self.y + row_index as i32, self.x, self.shape[row_index as usize]);
		}
	}
}

fn shoot_projectile(screen: &Window, target: &mut Target<'_>, max_y: i32, max_x: i32, y: i32, x: i32) -> bool {
	let straight_pitchfork = vec![
		"               /====»",
		"              /",
		"▓▓▓▓▓▓▓▓▓▓▓▓▓|======»",
		"              \\",
		"                \\====»"];

	let medium_angled_pitchfork = vec![
		"                  |==»",
		"                 /|",
		"                /|          |==»",
		"              //           /|",
		"              \\          /|",
		"               \\        /|",
		"                \\     /|",
		"          ▓▓▓▓▓▓  \\   ||",
		"     ▓▓▓▓▓▓         \\//",
		"▓▓▓▓▓▓"];

	let most_vertical_pitchfork = vec![
		"    ^",
		"   //             ^",
		"  ||             //",
		"  \\           ||",
		"   \\___      //",
		"    ▓▓  \\__||",
		"   ▓▓",
		"  ▓▓",
		" ▓▓",
		"▓▓"];


	let (v0y, v0x) = ((y as f32 * 19.6).sqrt(), (x / 2) as f32);
	let mut t = 0.0;
	loop {
		let (curr_y, curr_x) = (v0y * t + 0.5 * -9.8 * t * t, v0x * t);
		let (screen_y, screen_x) = ((max_y - curr_y as i32), curr_x as i32);

		{
			let close_to_target = |target_pos, projectile_pos|
				target_pos <= projectile_pos + 5 && target_pos >= projectile_pos - 5;
		
			if close_to_target(target.y, screen_y) && close_to_target(target.x, screen_x) {
				return true;
			}
		}

		let ascii_arrow = {
			let curr_angle = (curr_y / curr_x * 2.0).atan() * 180.0 / 3.1415926535;
			if curr_angle < 30.0 {&straight_pitchfork}
			else if curr_angle < 60.0 {&medium_angled_pitchfork}
			else {&most_vertical_pitchfork}};

		let mut row_index = 0;
		screen.clear();
		for y_coord in screen_y .. screen_y + ascii_arrow.len() as i32 {
			let row = ascii_arrow[row_index];
			screen.mvprintw(y_coord, screen_x, row);
			row_index += 1;
		}
		target.update(max_y);
		target.draw(&screen);
		screen.refresh();
		napms(1000 / FPS);
		t += 0.1;
		if (curr_y as i32) < 0 || (curr_x as i32) > max_x {break;}
	}
	false
}

fn main() {
	let screen = initscr();
	screen.nodelay(true);
	screen.keypad(true);
	cbreak();
	noecho();
	mousemask(ALL_MOUSE_EVENTS, std::ptr::null_mut());
	mouseinterval(0);
	let (max_y, max_x) = screen.get_max_yx();
	let mut target = Target {y: 0, x: max_x - max_x / 5, going_down: true, shape:
		vec!["/^^^^^^^^^^^\\",
			 "| _■_|╦|_■_ |",
			 " \\ ░░░║░░░░/",
			 "  \\░░░╩░░░/",
			 "   \\░░Ø░░/",
			 "    |(-)|"]};
	let (mut score, mut consecutive_fails) = (0, 0);
	loop {
		match screen.getch() {
			Some(Input::Character('q')) => {break;},
			Some(Input::KeyMouse) => {
				if let Ok(click) = getmouse() {
					if click.bstate == BUTTON1_PRESSED && click.x < max_x / 2 {
						if shoot_projectile(&screen, &mut target, max_y, max_x, max_y - click.y, click.x)
						{score += 1; consecutive_fails = 0;}
						else {score = 0; consecutive_fails += 1;}
					}
				}
			},
			_ => ()
		}
		target.update(max_y);	
		target.draw(&screen);
		for border_y in 0 .. max_y {screen.mvprintw(border_y, max_x / 2, "|");}

		let msg = format!("You have hit Milburn {} time{} in a row. |", score, if score == 1 {""} else {"s"});
		screen.mvprintw(0, 0, &msg);
		let score_msg_len = msg.len();
		screen.mvprintw((|| {if consecutive_fails > 2 {
				if MILBURN_LAUGHS {beep();}
				screen.mvprintw(1, 0, format!("Milburn says Ha Ha!{}|", " ".repeat(score_msg_len - 20)));
				return 2;} 1})(), 0, "_".repeat(score_msg_len - 1) + "|");

		screen.refresh();
		napms(1000 / FPS);
		screen.clear();
	}
	endwin();
}
