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
    use ink::prelude::vec::Vec;
    use TrabajoFinal::TrabajoFinalRef;
    use TrabajoFinal::Votante;
    use scale_info::prelude::string::String;
    
    #[ink(storage)]
    pub struct Reporte {
        trabajo_final: TrabajoFinalRef,
    }

    impl Reporte {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(trabajo_final_code_hash: Hash) -> Self {
            let trabajo_final = TrabajoFinalRef::new()
                .code_hash(trabajo_final_code_hash)
                .endowment(0)
                .salt_bytes(Vec::new())
                .instantiate();
        
            Self { trabajo_final }
        }
        
        #[ink(message)]
        pub fn test(&mut self) -> Result<u64, String> {
           match self.trabajo_final.obtener_datos_reporte(1) {
                Err(e) => Err(e),
                Ok(value) => {
                    Ok(value[0])
                }
           }
        }
        //Metodo privado para obtener los votantes de la eleccion y despues procesar la data
        fn obtener_votantes_eleccion(&mut self,eleccion_id:u64) -> Result<Vec<Votante>,String>{
            match self.trabajo_final.obtener_votantes_eleccion_por_id(eleccion_id){
                Ok(votantes_eleccion) => return Ok(votantes_eleccion),
                Err(mensaje) => return Err(mensaje)
            };   
        }
        fn calcular_participacion_eleccion(){

        }
        pub fn generar_reporte_votantes(&mut self,eleccion_id:u64){
            match self.obtener_votantes_eleccion(eleccion_id){

            }
        }
        pub fn generar_reporte_participacion(&mut self,eleccion_id:u64) -> Result<String,String>{
            let votantes = match self.obtener_votantes_eleccion(eleccion_id){
                Ok(votantes) => votantes,
                Err(mensaje) => return Err(mensaje)
            };
            let votos_emitidos = votantes.iter().filter(|votante| votante.get_voto()).count() as u32;
            let cant_votantes = votantes.len() as u32;

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