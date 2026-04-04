mod characters;   // Busca characters/mod.rs
mod playthrough;  // Busca playthrough/mod.rs

use characters::jugador::Jugador;   // characters → jugador → Jugador
use playthrough::combate::pelear;   // playthrough → combate → pelear

fn main() {
    let mut player = Jugador::nuevo("Lucas", 25);
    let mut goblin = Jugador::nuevo("Goblin", 10);

    println!("--- Inicio ---");
    println!("{}: {} HP", player.nombre, player.vida);
    println!("{}: {} HP", goblin.nombre, goblin.vida);

    println!("--- Combate ---");
    pelear(&player, &mut goblin);
    pelear(&player, &mut goblin);
    pelear(&goblin, &mut player);

    println!("--- Resultado ---");
    if player.esta_vivo() {
        println!("{} sobrevivió con {} HP", player.nombre, player.vida);
    } else {
        println!("{} fue derrotado", player.nombre);
    }
}
