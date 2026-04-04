#[derive(Debug)]
pub struct Jugador {
    pub nombre: String,
    pub vida: i32,
    danio: i32, // Privado — solo este módulo lo ve
}

impl Jugador {
    pub fn nuevo(nombre: &str, danio: i32) -> Jugador {
        Jugador {
            nombre: String::from(nombre),
            vida: 100,
            danio,
        }
    }

    pub fn esta_vivo(&self) -> bool {
        self.vida > 0
    }

    pub fn get_danio(&self) -> i32 {
        self.danio
    }

    pub fn recibir_danio(&mut self, cantidad: i32) {
        self.vida -= cantidad;
        if self.vida < 0 {
            self.vida = 0;
        }
    }
}
