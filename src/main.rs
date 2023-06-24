struct Coordinate(i8, i8);
#[derive(Debug)]
struct Delver {
    name: String,
    exploriness: f64,
    fightiness: f64,
    hp:i8
}
struct Room {
    coord:Coordinate
}
struct State {
    delvers:Vec<Delver>,
    rooms: Vec<Room>,
    last_log_message:String,
}
fn main() {
    let delvers = Vec::new();
    let rooms = Vec::new();
    let last_log_message = String::from("Example Message");
    let mut game_state = State{delvers,rooms,last_log_message};
    
    let name = String::from("Fighter");
    let exploriness = 0.2;
    let fightiness = 0.8;
    let hp = 5;
    let c = Delver {name, exploriness, fightiness, hp};

    game_state.delvers.push(c);

    let name = String::from("Rogue");
    let exploriness = 0.8;
    let fightiness = 0.2;
    let hp = 5;
    let c = Delver {name, exploriness, fightiness, hp};
    game_state.delvers.push(c);

}
