use crate::characters::jugador::Jugador;

pub fn pelear(atacante: &Jugador, defensor: &mut Jugador) {
    let danio = atacante.get_danio();
    println!("{} ataca a {} por {} de daño!",
        atacante.nombre, defensor.nombre, danio);
    defensor.recibir_danio(danio);
    println!("  {} tiene {} HP", defensor.nombre, defensor.vida);
}
