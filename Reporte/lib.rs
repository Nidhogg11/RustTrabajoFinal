#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod Reporte {
    use TrabajoFinal::TrabajoFinalRef;

    use core::ops::Mul;
    use ink::prelude::vec::Vec;
    use scale_info::prelude::string::String;

    #[ink(storage)]
    pub struct Reporte {
        trabajo_final: TrabajoFinalRef,
    }

    #[ink(impl)]
    impl Reporte {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor, payable)]
        pub fn new(version: u32, trabajo_final_hash: Hash) -> Self {
            let salt = version.to_le_bytes();
            let trabajo_final = TrabajoFinalRef::new()
                .endowment(0)
                .code_hash(trabajo_final_hash)
                .salt_bytes(salt)
                .instantiate();
            Self { trabajo_final }
        }

        // ➢ Reporte de Registro de Votantes: Detalla los votantes registrados y aprobados para
        // una determinada elección.
        #[ink(message)]
        pub fn reporte_de_votantes_por_eleccion(
            &mut self,
            id_eleccion: u64,
        ) -> Result<Vec<(AccountId, String, String, String)>, String> {
            self.reporte_de_votantes_por_eleccion_privado(id_eleccion)
        }
        pub fn reporte_de_votantes_por_eleccion_privado(
            &mut self,
            id_eleccion: u64,
        ) -> Result<Vec<(AccountId, String, String, String)>, String> {
            let datos_votantes = match self
                .trabajo_final
                .obtener_votantes_eleccion_por_id(id_eleccion)
            {
                Err(msg) => return Err(msg),
                Ok(datos) => datos,
            };

            Ok(datos_votantes
                .iter()
                .map(|datos_votante| {
                    let datos_usuario = self
                        .trabajo_final
                        .obtener_informacion_usuario(datos_votante.0)
                        .unwrap_or_default();
                    (
                        datos_votante.0.clone(),
                        datos_usuario.0,
                        datos_usuario.1,
                        datos_usuario.2,
                    )
                })
                .collect())
        }

        // ➢ Reporte de Participación: Indica la cantidad de votos emitidos y el porcentaje de
        // participación, una vez cerrada la elección.
        #[ink(message)]
        pub fn reporte_de_participacion_por_eleccion(
            &mut self,
            id_eleccion: u64,
        ) -> Result<(u32, u32), String> {
            self.reporte_de_participacion_por_eleccion_privado(id_eleccion)
        }
        pub fn reporte_de_participacion_por_eleccion_privado(
            &mut self,
            id_eleccion: u64,
        ) -> Result<(u32, u32), String> {
            let datos_votantes = match self
                .trabajo_final
                .obtener_votantes_eleccion_por_id(id_eleccion)
            {
                Err(msg) => return Err(msg),
                Ok(datos) => datos,
            };

            let cantidad_votantes = datos_votantes.len() as u32;
            let cantidad_votantes_voto_efectivo =
                datos_votantes.iter().filter(|vot| vot.1).count() as u32;
            let porcentaje_participacion =
                (cantidad_votantes_voto_efectivo.div_euclid(cantidad_votantes)).mul(100);
            Ok((cantidad_votantes_voto_efectivo, porcentaje_participacion))
        }

        // ➢ Reporte de Resultado:: Muestra el número de votos recibidos por cada candidato y
        // los resultados finales, una vez cerrada la elección. Este reporte deberá mostrar de
        // manera descendente los votos, donde el primer candidato será el ganador de la
        // elección.
        #[ink(message)]
        pub fn reporte_de_resultado_por_eleccion(
            &mut self,
            id_eleccion: u64,
        ) -> Result<
            (
                (AccountId, String, String, String, u32),
                Vec<(AccountId, String, String, String, u32)>,
            ),
            String,
        > {
            self.reporte_de_resultado_por_eleccion_privado(id_eleccion)
        }
        pub fn reporte_de_resultado_por_eleccion_privado(
            &mut self,
            id_eleccion: u64,
        ) -> Result<
            (
                (AccountId, String, String, String, u32),
                Vec<(AccountId, String, String, String, u32)>,
            ),
            String,
        > {
            let mut datos_candidatos = match self
                .trabajo_final
                .obtener_candidatos_eleccion_por_id(id_eleccion)
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

            Ok((candidatos[0].clone(), candidatos))
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        // use ink_e2e::*;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_Reporte<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let trabajo_final_hash = client
                .upload("TrabajoFinal", &ink_e2e::alice(), None)
                .await
                .expect("uploading `TrabajoFinal` failed")
                .code_hash;

            let constructor_reporte = ReporteRef::new(
                0, //version
                trabajo_final_hash,
            );
            println!("{:#?}", trabajo_final_hash);

            let reporte = client
                .instantiate("Reporte", &ink_e2e::alice(), constructor_reporte, 0, None)
                .await
                .expect("instantiate failed");

            println!("{:#?}", reporte);
            let reporte_acc_id = reporte.account_id;
            println!("{:#?}", reporte_acc_id);
            // when
            // let get = build_message::<ReporteRef>(reporte_acc_id.clone())
            //     .call(|contract| contract.get());
            // let value = client
            //     .call_dry_run(&ink_e2e::bob(), &get, 0, None)
            //     .await
            //     .return_value();
            // assert_eq!(value, 1234);
            // let change =
            //     build_message::<ReporteRef>(reporte_acc_id.clone())
            //         .call(|contract| contract.change(6));
            // let _ = client
            //     .call(&ink_e2e::bob(), change, 0, None)
            //     .await
            //     .expect("calling `change` failed");

            // // then
            // let get = build_message::<ReporteRef>(reporte_acc_id.clone())
            //     .call(|contract| contract.get());
            // let value = client
            //     .call_dry_run(&ink_e2e::bob(), &get, 0, None)
            //     .await
            //     .return_value();
            // assert_eq!(value, 1234 + 6);

            // // when
            // let switch =
            //     build_message::<ReporteRef>(reporte_acc_id.clone())
            //         .call(|contract| contract.switch());
            // let _ = client
            //     .call(&ink_e2e::bob(), switch, 0, None)
            //     .await
            //     .expect("calling `switch` failed");
            // let change =
            //     build_message::<ReporteRef>(reporte_acc_id.clone())
            //         .call(|contract| contract.change(3));
            // let _ = client
            //     .call(&ink_e2e::bob(), change, 0, None)
            //     .await
            //     .expect("calling `change` failed");

            // // then
            // let get = build_message::<ReporteRef>(reporte_acc_id.clone())
            //     .call(|contract| contract.get());
            // let value = client
            //     .call_dry_run(&ink_e2e::bob(), &get, 0, None)
            //     .await
            //     .return_value();
            // assert_eq!(value, 1234 + 6 - 3);

            Ok(())
        }
    }
}
