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
        
        /// Utilizado por todos los usuarios.
        /// Obtiene la lista de votantes para una elección específica.
        /// Parametors:
        ///     id_eleccion: u64: ID de la elección.
        /// Retorno:
        ///     Result<Vec<(AccountId, String, String, String)>, String>: Vector con el ID de cada votante y su información detallada, o un mensaje de error.
        /// Descripción:
        /// La función recupera la lista de votantes de una elección dada por su ID (`id_eleccion`). Llama a una función privada
        /// para obtener los datos y luego añade información detallada sobre cada votante. Retorna un vector de tuplas con 
        /// el `AccountId`, nombre, dirección y otros detalles del votante, o un mensaje de error en caso de fallo.
        #[ink(message)]
        pub fn reporte_de_votantes_por_eleccion(&mut self, id_eleccion: u64) -> Result< Vec<(AccountId, String, String, String)>, String > {
            self.reporte_de_votantes_por_eleccion_privado(id_eleccion)
        }
        pub fn reporte_de_votantes_por_eleccion_privado(&mut self, id_eleccion: u64) -> Result< Vec<(AccountId, String, String, String)>, String > {
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

        /// Obtiene la participación en una elección específica.
        /// Parametros:
        ///     id_eleccion: u64: ID de la elección.
        /// Retorno:
        ///     Result<(u32, u32), String>: Una tupla con la cantidad de votantes efectivos y el porcentaje de participación, o un mensaje de error.
        /// Descripción:
        /// La función recupera la participación en una elección indicada por `id_eleccion`. Llama a una función privada
        /// para obtener los datos de los votantes. Calcula el número de votantes que participaron efectivamente y el 
        /// porcentaje de participación. Devuelve estos valores en una tupla, o un mensaje de error si falla.
        #[ink(message)]
        pub fn reporte_de_participacion_por_eleccion(&mut self, id_eleccion: u64) -> Result<(u32, u32), String> {
            self.reporte_de_participacion_por_eleccion_privado(id_eleccion)
        }
        pub fn reporte_de_participacion_por_eleccion_privado(&mut self, id_eleccion: u64) -> Result<(u32, u32), String> {
            let datos_votantes = match self.trabajo_final.obtener_votantes_eleccion_por_id(id_eleccion) {
                Err(msg) => return Err(msg),
                Ok(datos) => datos 
            };

            let cantidad_votantes = datos_votantes.len() as u32;
            let cantidad_votantes_voto_efectivo = datos_votantes.iter().filter(|vot: &&(ink::primitives::AccountId, bool)| vot.1).count() as u32;
            let porcentaje_participacion = cantidad_votantes_voto_efectivo.mul(100).div_ceil(cantidad_votantes);
            Ok( (cantidad_votantes_voto_efectivo, porcentaje_participacion) )
        }

        /// Permite obtener un reporte los datos de un candidato en particular dentro de una elección específica.
        /// Parámetros
        ///    eleccion_id (u64): El ID de la elección de la cual se quiere obtener la información del candidato.
        ///
        /// Retorno
        /// Result< ((AccountId, String, String, String, u32), Vec<(AccountId, String, String, String, u32)>), String>:
        /// Los datos del ganador de la eleccion si no resulta en empate y un Vector ordenado con: ID de cada candidato, Nombre, Apellido, DNI y su total de votos,
        /// o un mensaje de error
        #[ink(message)]
        pub fn reporte_de_resultado_por_eleccion(&mut self, id_eleccion: u64) -> Result<(Option<(AccountId, String, String, String, u32)>, Vec<(AccountId, String, String, String, u32)>), String> {
            self.reporte_de_resultado_por_eleccion_privado(id_eleccion)
        }
        pub fn reporte_de_resultado_por_eleccion_privado(&mut self, id_eleccion: u64) -> Result<(Option<(AccountId, String, String, String, u32)>, Vec<(AccountId, String, String, String, u32)>), String> {
            let mut datos_candidatos = match self.trabajo_final.obtener_candidatos_eleccion_por_id(id_eleccion)
            {
                Err(msg) => return Err(msg),
                Ok(datos) => datos,
            };

            // Ordenar datos_candidatos por la cantidad de votos (descendente)
            datos_candidatos.sort_by(|a, b| b.1.cmp(&a.1));

            let candidatos: Vec<(ink::primitives::AccountId, String, String, String, u32)> =
                datos_candidatos
                    .iter()
                    .map(|datos_candidato| {
                        let datos_usuario = self
                            .trabajo_final
                            .obtener_informacion_usuario(datos_candidato.0)
                            .unwrap_or_default();
                        (
                            datos_candidato.0.clone(),
                            datos_usuario.0,
                            datos_usuario.1,
                            datos_usuario.2,
                            datos_candidato.1,
                        )
                    })
                    .collect();

            if candidatos.len() >= 2 {
                if candidatos[0].4 == candidatos[1].4 {
                    return Ok((None, candidatos));
                }
            }
            Ok((Some(candidatos[0].clone()), candidatos))
        }
    }


    #[cfg(test)]
    // #[cfg(all(test, feature = "tests"))]
    mod tests {
        use super::*;
        use ink::env::test::{
            default_accounts, get_account_balance, recorded_events,
            DefaultAccounts, EmittedEvent
        };
        use ink::env::DefaultEnvironment;

        use TrabajoFinal::StructTrabajoFinal;
    
        // Returns accounts that are pre-seeded in the test database.
        // We can use them as authors for transactions.
        fn get_default_test_accounts() -> DefaultAccounts<ink::env::DefaultEnvironment> {
            default_accounts::<ink::env::DefaultEnvironment>()
        }

        fn set_caller(caller: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(caller);
        }

        fn setup_sistema_elecciones_vacio() -> StructTrabajoFinal {
            let accounts = get_default_test_accounts();
            let alice = accounts.alice;
            let charlie = accounts.charlie;
            set_caller(alice);

            StructTrabajoFinal::new()
        }
    }
}