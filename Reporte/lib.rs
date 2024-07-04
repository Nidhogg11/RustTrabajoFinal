#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod Reporte {
    use core::ops::Mul;

    use ink::prelude::vec::Vec;
    use TrabajoFinal::TrabajoFinalRef;
    use scale_info::prelude::string::String;
    
    #[ink(storage)]
    pub struct Reporte {
        trabajo_final: TrabajoFinalRef,
    }

    #[ink(impl)]
    impl Reporte {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(trabajo_final: TrabajoFinalRef) -> Self {
            Self { trabajo_final }
        }
        
        // ➢ Reporte de Registro de Votantes: Detalla los votantes registrados y aprobados para
        // una determinada elección.
        #[ink(message)]
        pub fn reporte_de_votantes_por_eleccion(&mut self, id_eleccion: u64) -> Result< Vec<(AccountId, String, String, String)>, String > {
            let datos_votantes = match self.trabajo_final.obtener_votantes_eleccion_por_id(id_eleccion) {
               Err(msg) => return Err(msg),
               Ok(datos) => datos 
            };

            Ok(
                datos_votantes
                .iter()
                .map( |datos_votante| {
                    let datos_usuario = self.trabajo_final.obtener_informacion_usuario(datos_votante.0).unwrap_or_default();
                    (datos_votante.0.clone(), datos_usuario.0, datos_usuario.1, datos_usuario.2)
                })
                .collect()
            )
        }

        // ➢ Reporte de Participación: Indica la cantidad de votos emitidos y el porcentaje de
        // participación, una vez cerrada la elección.
        #[ink(message)]
        pub fn reporte_de_participacion_por_eleccion(&mut self, id_eleccion: u64) -> Result<(u32, u32), String> {
            let datos_votantes = match self.trabajo_final.obtener_votantes_eleccion_por_id(id_eleccion) {
                Err(msg) => return Err(msg),
                Ok(datos) => datos 
            };

            let cantidad_votantes = datos_votantes.len() as u32;
            let cantidad_votantes_voto_efectivo = datos_votantes.iter().filter(|vot| vot.1).count() as u32;
            let porcentaje_participacion = (cantidad_votantes_voto_efectivo.div_euclid(cantidad_votantes)).mul(100);
            Ok( (cantidad_votantes_voto_efectivo, porcentaje_participacion) )
        }

        // ➢ Reporte de Resultado:: Muestra el número de votos recibidos por cada candidato y
        // los resultados finales, una vez cerrada la elección. Este reporte deberá mostrar de
        // manera descendente los votos, donde el primer candidato será el ganador de la
        // elección.
        #[ink(message)]
        pub fn reporte_de_resultado_por_eleccion(&mut self, id_eleccion: u64) -> Result< ((AccountId, String, String, String, u32), Vec<(AccountId, String, String, String, u32)>), String> {
            let mut datos_candidatos = match self.trabajo_final.obtener_candidatos_eleccion_por_id(id_eleccion) {
                Err(msg) => return Err(msg),
                Ok(datos) => datos 
            };
            
            // Ordenar datos_candidatos por la cantidad de votos (descendente)
            datos_candidatos.sort_by(|a, b| b.1.cmp(&a.1));

             let candidatos: Vec<(ink::primitives::AccountId, String, String, String, u32)> = datos_candidatos
             .iter()
             .map( |datos_candidato| {
                 let datos_usuario = self.trabajo_final.obtener_informacion_usuario(datos_candidato.0).unwrap_or_default();
                 (datos_candidato.0.clone(), datos_usuario.0, datos_usuario.1, datos_usuario.2, datos_candidato.1)
             })
             .collect();

             Ok(( candidatos[0].clone(), candidatos ))
        }
    }
}