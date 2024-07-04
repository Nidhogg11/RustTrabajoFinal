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
        fn get_default_test_accounts(
        ) -> DefaultAccounts<ink::env::DefaultEnvironment> {
            default_accounts::<ink::env::DefaultEnvironment>()
        }

        // Sets caller returned by the next `Self::env().caller()` method call
        // in the contract.
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
    
        // #[test]
        // fn constructor_works() {
        //     let sistema_elecciones = StructTrabajoFinal::new();
        //     assert_eq!(sistema_elecciones.\, 0);
        // }

        // #[test]
        // fn test_es_generador_reportes() {
        //     let mut sistema_elecciones = setup_sistema_elecciones_vacio();
        //     let error = "No es el generador de reportes!";
        //     assert_ne!(error.to_string(), "No es el generador de reportes!");
        // }
    
        // #[test]
        // fn test_no_es_generador_reportes() {
        //     let error = "No es el generador de reportes!";
        //     assert_eq!(error.to_string(), "No es el generador de reportes!");
        // }

        // #[test]
        // fn test_contiene_usuario_pendiente() {
        //     let mut eleccion = setup_eleccion();
        //     let accounts = default_accounts::<DefaultEnvironment>();//uentas predeterminadas utilizadas para tests
            
        //     eleccion.usuarios_pendientes.push((accounts.alice, TIPO_DE_USUARIO::VOTANTE));
            
        //     assert!(eleccion.contiene_usuario_pendiente(accounts.alice));
        //     assert!(!eleccion.contiene_usuario_pendiente(accounts.bob));
        // }
    
        // #[test]
        // fn test_existe_candidato() {
        //     let mut eleccion = setup_eleccion();
        //     let accounts = default_accounts::<DefaultEnvironment>();
            
        //     eleccion.candidatos.push(CandidatoConteo {
        //         id: accounts.alice,
        //         candidato_id: 1,
        //         votos_totales: 0,
        //     });
    
        //     assert!(eleccion.existe_candidato(1));
        //     assert!(!eleccion.existe_candidato(2));
        // }
    }
}