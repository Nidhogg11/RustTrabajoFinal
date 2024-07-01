#![cfg_attr(not(feature = "std"), no_std, no_main)]


// Generación de Reportes en otro contrato
// el sistema tiene que poder generar varios reportes desde otro contrato con los permisos
// pertinentes:
//     ➢ Reporte de Registro de Votantes: Detalla los votantes registrados y aprobados para
//     una determinada elección.
//     ➢ Reporte de Participación: Indica la cantidad de votos emitidos y el porcentaje de
//     participación, una vez cerrada la elección.
//     ➢ Reporte de Resultado:: Muestra el número de votos recibidos por cada candidato y
//     los resultados finales, una vez cerrada la elección. Este reporte deberá mostrar de
//     manera descendente los votos, donde el primer candidato será el ganador de la
//     elección.


#[ink::contract]
mod Reporte {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use scale_info::prelude::format;
    use ink::prelude::string::ToString;

    use TrabajoFinal::TrabajoFinalRef;
    
    #[ink(storage)]
    pub struct Reporte {
        trabajo_final: TrabajoFinalRef,
    }

    impl Reporte {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(_trabajo_final: TrabajoFinalRef) -> Self {
            Self { trabajo_final: _trabajo_final }
        }
        
        #[ink(message)]
        pub fn test(&mut self, id_eleccion: u64) -> Result<String, String> {
           match self.trabajo_final.obtener_datos_reporte(1) {
               Err(e) => Err(e),
               Ok(value) => {
                let suma = value.iter().min().unwrap_or(&10000).to_string();
                Ok(String::from("Tremendos datos: ") + &suma.as_str())
            }
           }
        }

        // #[ink(message)]
        // pub fn reporte_de_votantes_por_eleccion(&self, id_eleccion: u64) -> Result<String, String> {
        //    Ok(String::from("LOREM"))
        // }

        // #[ink(message)]
        // pub fn reporte_de_participacion_por_eleccion(&self, id_eleccion: u64) -> Result<String, String> {
        //    Ok(String::from("LOREM"))
        // }

        // #[ink(message)]
        // pub fn reporte_de_resultado_por_eleccion(&self, id_eleccion: u64) -> Result<String, String> {
        //    Ok(String::from("LOREM"))
        // }
    }
}