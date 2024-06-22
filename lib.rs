#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod TrabajoFinal {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;

    enum ERRORES
    {
        NO_ES_ADMINISTRADOR,
        USUARIO_NO_REGISTRADO,
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout,PartialEq,Clone))]
    pub enum ESTADO_ELECCION
    {
        CERRADA,
        ABIERTA,
        INICIADA,
        FINALIZADA
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout,PartialEq,Clone))]
    pub enum TIPO_DE_USUARIO
    {
        VOTANTE,
        CANDIDATO
    }

    impl ERRORES
    {
        fn to_string(&self) -> String
        {
            match self 
            {
                ERRORES::NO_ES_ADMINISTRADOR => String::from("No eres el administrador."),
                ERRORES::USUARIO_NO_REGISTRADO => String::from("No estás registrado en el sistema. Espera a que te acepten en el mismo o realiza la solicitud.")
            }
        }
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    struct Usuario
    {
        id:AccountId,
        nombre:String,
        apellido:String,
        dni:String,
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    struct Votante
    {
        id:AccountId,
        voto_emitido:bool,
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    struct CandidatoConteo
    {
        id:AccountId,
        candidato_id:u32,
        votos_totales:u32,
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    struct Eleccion
    {
        id:u32,
        candidatos:Vec<CandidatoConteo>,
        votantes:Vec<Votante>,
        usuarios_rechazados:Vec<AccountId>,
        usuarios_pendientes:Vec<(AccountId,TIPO_DE_USUARIO)>,
        estado: ESTADO_ELECCION, 
        fecha_inicio:u64,
        fecha_final:u64,
    }

    impl Eleccion
    {
        fn contiene_usuario(&self, id: AccountId) -> bool {
            self.usuarios_pendientes.iter().any(|(usuario_id, _tipo)| *usuario_id == id)
        }

        ///Usado por el administrador.
        ///Revisa el prier usuario pendiente.
        ///Lo envia al Vec candidato si es candidato, o votante en caso contrario.
        pub fn procesar_siguiente_usuario_pendiente(&mut self, aceptar_usuario:bool) -> Result<String, String>
        {
            let sig_usuario = self.usuarios_pendientes.first();
            if sig_usuario.is_none() { return Err(String::from("No hay usuarios pendientes.")); }

            let (usuario, tipo) = self.usuarios_pendientes.remove(0);
            if aceptar_usuario { 
                match tipo {
                    TIPO_DE_USUARIO::VOTANTE =>{
                    self.votantes.push(Votante{
                        id:usuario,
                        voto_emitido:false,
                    });

                   },
                   TIPO_DE_USUARIO::CANDIDATO=>{
                    let candidato_id:u32 = 1 + self.candidatos.len() as u32;
                    self.candidatos.push(CandidatoConteo{
                        id:usuario,
                        candidato_id:candidato_id,
                        votos_totales:0,
                    });
                   },
                   }
                return Ok(String::from("Usuario agregado exitosamente."));
            }
            else{
                self.usuarios_rechazados.push(usuario);
                return Ok(String::from("Usuario rechazado exitosamente."));
            }
        }
    }

    #[ink(storage)]
    pub struct TrabajoFinal {
        administrador:AccountId,
        registro_activado:bool,
        usuarios:Vec<Usuario>,
        usuarios_pendientes:Vec<Usuario>,
        usuarios_rechazados:Vec<Usuario>,
        elecciones:Vec<Eleccion>,
    }

    impl TrabajoFinal {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { 
                administrador: Self::env().caller(),
                registro_activado: false,
                usuarios: Vec::new(),
                usuarios_pendientes: Vec::new(),
                usuarios_rechazados: Vec::new(),
                elecciones: Vec::new(),
            }
        }

        fn es_usuario_registrado(&self, id: AccountId) -> bool {
            let id = self.env().caller();
            self.usuarios.iter().any(|usuario| usuario.id == id)
        }
        fn es_usuario_pendiente(&self, id: AccountId) -> bool {
            let id=self.env().caller();
            self.usuarios_pendientes.iter().any(|usuario| usuario.id == id)
        }
        fn es_usuario_rechazado(&self, id: AccountId) -> bool {
            let id=self.env().caller();
            self.usuarios_rechazados.iter().any(|usuario| usuario.id == id)
        }

        fn obtener_usuario_por_id(&mut self, id_usuario: AccountId) -> Option<&Usuario> {
            if self.es_usuario_registrado(id_usuario) { 
                    return self.usuarios.iter().find(|user| user.id == id_usuario);
            }
            return None;
        }
        fn obtener_usuario_pendiente_por_id(&mut self, id_usuario: AccountId) -> Option<&Usuario> {
            if self.es_usuario_pendiente(id_usuario) { 
                return self.usuarios_pendientes.iter().find(|user| user.id == id_usuario);
            }
            return None;
        }
        fn obtener_usuario_rechazado_por_id(&mut self, id_usuario: AccountId) -> Option<&Usuario> {
            if self.es_usuario_rechazado(id_usuario) { 
                return self.usuarios_rechazados.iter().find(|user| user.id == id_usuario);
            }
            return None;
        }

        fn existe_eleccion(&self, eleccion_id:u32) -> bool
        {
            if eleccion_id >= 1 && eleccion_id <= self.elecciones.len() as u32 {
                return true;
            }
            return false;
        }

        fn obtener_eleccion_por_id(&mut self, eleccion_id:u32) -> Option<&mut Eleccion>
        {
            if self.existe_eleccion(eleccion_id) {
                    return Some(&mut self.elecciones[(eleccion_id - 1) as usize]);
            }
            return None;
        }

        fn es_administrador(&self) -> bool
        {
            self.env().caller() == self.administrador
        }


        fn validar_estado_eleccion(&mut self,eleccion_id:u32,block_timestamp:u64,id_usuario:AccountId) -> Result<&mut Eleccion,String>{
            let option_eleccion = self.obtener_eleccion_por_id(eleccion_id);
            if option_eleccion.is_none() { return Err(String::from("No existe una elección con ese id.")); }
            
            let eleccion = option_eleccion.unwrap();
            if eleccion.contiene_usuario(id_usuario) { return Err(String::from("Ya está registrado en la elección.")); }
            
            if eleccion.estado == ESTADO_ELECCION::ABIERTA || eleccion.fecha_inicio < block_timestamp {
                return Err(String::from("La votación en la elección ya comenzó, no te puedes registrar."));
            }
            if eleccion.fecha_final < block_timestamp {
                return Err(String::from("La elección ya finalizó, no te puedes registrar."));
            }
            Ok(eleccion)
        }


    // ===================================================================================================
    // =========================creacion y administracion de estados de elecciones========================
    // ===================================================================================================

        /// Utilizado por un administrador.
        /// Crea una elección colocando fecha de inicio y final.
        #[ink(message)]
        pub fn crear_eleccion(&mut self, fecha_inicial:String, fecha_final:String) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }

            let fecha_inicial_milisegundos = chrono::NaiveDateTime::parse_from_str(&fecha_inicial, "%d-&m-&Y &H:&M");
            if fecha_inicial_milisegundos.is_err() {
                return Err(String::from("Error en el formato de la fecha inicial. Formato: dd-mm-YYYY hh:mm"));
            }
            let fecha_final_milisegundos = chrono::NaiveDateTime::parse_from_str(&fecha_final, "%d-&m-&Y &H:&M");
            if fecha_final_milisegundos.is_err() {
                return Err(String::from("Error en el formato de la fecha final. Formato: dd-mm-YYYY hh:mm"));
            }

            let eleccion_id = (self.elecciones.len() + 1) as u32;
            let eleccion = Eleccion {
                id: eleccion_id,
                candidatos: Vec::new(),
                votantes: Vec::new(),
                usuarios_pendientes: Vec::new(),
                usuarios_rechazados: Vec::new(),
                estado: ESTADO_ELECCION::CERRADA,
                fecha_inicio: fecha_inicial_milisegundos.unwrap().and_utc().timestamp_millis() as u64,
                fecha_final: fecha_final_milisegundos.unwrap().and_utc().timestamp_millis() as u64,
            };
            self.elecciones.push(eleccion);

            return Ok(String::from("Eleccion creada exitosamente. Id de la elección: ") + &eleccion_id.to_string());
        }

        /// Utilizado por un administrador.
        /// cierra una elección colocando su estado en CERRADO (estado anterior al INICIADA).
        #[ink(message)]
        pub fn cerrar_eleccion(&mut self, eleccion_id: u32) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            let eleccion_option = self.obtener_eleccion_por_id(eleccion_id);
            match eleccion_option {
                Some(eleccion) => {
                    if eleccion.estado == ESTADO_ELECCION::CERRADA {
                        return Err(String::from("La eleccion ya se encuantra en el estado correspondiente!"))
                    }
                    eleccion.estado = ESTADO_ELECCION::CERRADA;
                    return Ok(String::from("Eleccion cerrada exitosamente. Id de la elección: ") + &eleccion.id.to_string());
                },
                None => return Err(String::from("La eleccion enviada no existe!")),
            };
        }
        /// Utilizado por un administrador.
        /// cierra una elección colocando su estado en CERRADO (estado anterior al INICIADA).
        #[ink(message)]
        pub fn abrir_eleccion(&mut self, eleccion_id: u32) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            let eleccion_option = self.obtener_eleccion_por_id(eleccion_id);
            match eleccion_option {
                Some(eleccion) => {
                    if eleccion.estado == ESTADO_ELECCION::CERRADA {
                        return Err(String::from("La eleccion ya se encuantra en el estado correspondiente!"))
                    }
                    eleccion.estado = ESTADO_ELECCION::CERRADA;
                    return Ok(String::from("Eleccion cerrada exitosamente. Id de la elección: ") + &eleccion.id.to_string());
                },
                None => return Err(String::from("La eleccion enviada no existe!")),
            };
        }
        /// Utilizado por un administrador.
        /// cierra una elección colocando su estado en CERRADO (estado anterior al INICIADA).
        #[ink(message)]
        pub fn iniciar_eleccion(&mut self, eleccion_id: u32) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            let eleccion_option = self.obtener_eleccion_por_id(eleccion_id);
            match eleccion_option {
                Some(eleccion) => {
                    if eleccion.estado == ESTADO_ELECCION::CERRADA {
                        return Err(String::from("La eleccion ya se encuantra en el estado correspondiente!"))
                    }
                    eleccion.estado = ESTADO_ELECCION::CERRADA;
                    return Ok(String::from("Eleccion cerrada exitosamente. Id de la elección: ") + &eleccion.id.to_string());
                },
                None => return Err(String::from("La eleccion enviada no existe!")),
            };
        }
        /// Utilizado por un administrador.
        /// cierra una elección colocando su estado en CERRADO (estado anterior al INICIADA).
        #[ink(message)]
        pub fn finalizar_eleccion(&mut self, eleccion_id: u32) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            let eleccion_option = self.obtener_eleccion_por_id(eleccion_id);
            match eleccion_option {
                Some(eleccion) => {
                    if eleccion.estado == ESTADO_ELECCION::CERRADA {
                        return Err(String::from("La eleccion ya se encuantra en el estado correspondiente!"))
                    }
                    eleccion.estado = ESTADO_ELECCION::CERRADA;
                    return Ok(String::from("Eleccion cerrada exitosamente. Id de la elección: ") + &eleccion.id.to_string());
                },
                None => return Err(String::from("La eleccion enviada no existe!")),
            };
        }

        #[ink(message)]
        pub fn obtener_ids_elecciones(&self) -> Vec<u32>
        {
            self.elecciones.iter().map(|eleccion| eleccion.id).collect()
        }

    // ====================================================================
    // ====================================================================
    // ====================================================================

        
        // Utilizado por los usuarios registrados en el sistema para poder ingresar a una elección.
        // Un usuario registrado y que no está registrado en la elección puede ingresar a la misma como candidato o votante.
        // Estos no pueden ingresar a la misma si esta ya comenzó su periodo de votación o ya terminó la elección.
        // Para ingresar como candidato es necesario una candidatura. 
        #[ink(message)]
        pub fn ingresar_a_eleccion(&mut self, eleccion_id:u32, tipo:TIPO_DE_USUARIO) -> Result<String, String>
        {
            // es usuario valido en el sistema (no esta pendiente de aprobacion y no esta rechazado)
            // no está registrado en la elección
            // el estado de la eleccion es PENDIENTE
            let id = self.env().caller();
            if !self.es_usuario_registrado(id) { return Err(ERRORES::USUARIO_NO_REGISTRADO.to_string()); }

            let block_timestamp = self.env().block_timestamp();
            let result = self.validar_estado_eleccion(eleccion_id, block_timestamp, id);
            let eleccion = match result {
                Ok(eleccion) => eleccion,
                Err(mensaje) => return Err(mensaje)
            };
            //Validar que un usuario que ya ha sido rechazado en la misma eleccion no intente volver a ponerse como pendiente 
            if eleccion.usuarios_rechazados.contains(&id) {return Err("Ya has sido rechazado no puedes ingresar a la eleccion".to_string())}
            
            if eleccion.contiene_usuario(id){return Err("No puedes ingresar dos veces a la misma eleccion".to_string())} // esto no esta contemplado ya en el validar eleccion?

            eleccion.usuarios_pendientes.push((id,tipo));

            return Ok(format!("Ingresó a la elección correctamente Pendiente de aprobacion del Administrador"));
              
        }

        /// Utilizado por los usuarios registrados en el sistema y que están en la elección como votantes.
        /// Si el usuario ya emitió su voto, no puede volver a votar en la misma elección.
        /// Si el usuario no es votante, no puede votar.
        /// Si el periodo de la votación no comenzó o terminó, no puede votar.
        // #[ink(message)]
        // pub fn votar_a_candidato(&mut self, eleccion_id:u32, candidato_id:u32) -> Result<String, String>
        // {
        //     todo!()
        // }

        /// Utilizado por los usuarios registrados en el sistema y que están en la elección ingresada.
        /// Se utiliza para poder obtener información de algún candidato en específico.
        /// Las IDs de los candidatos van de 1 a N.
        // pub fn obtener_informacion_candidato(&self, eleccion_id:u32, candidato_id:u32) -> Result<String, String>
        // {
        //     todo!()
        // }


    // ===================================================================================================
    // =========================creacion y administracion de usuarios========================
    // ===================================================================================================

        /// Utilizado por los usuarios para poder registrarse en el sistema.
        /// Luego de registrarse queda pendiente de aceptación por parte de un Administrador.
        /// Si tu registro es rechazado, no podrás volver a intentar registrarte.
        #[ink(message)]
        pub fn registrarse(&mut self, nombre:String, apellido:String, dni:String) -> Result<String, String>
        {
            if !self.registro_activado { return Err(String::from("El registro todavía no está activado.")); }
            if self.es_administrador() { return Err(String::from("Eres el administrador, no puedes registrarte.")); }

            let id = self.env().caller();
            if self.es_usuario_rechazado(id) {  return Err(String::from("Tu solicitud de registro ya fue rechazada.")); }
            if self.es_usuario_registrado(id) { return Err(String::from("Ya estás registrado como usuario.")); }
            if self.es_usuario_pendiente(id) { return Err(String::from("Ya estás en la cola de usuarios pendientes.")); }

            let usuario = Usuario { id, nombre, apellido, dni };
            self.usuarios_pendientes.push(usuario);
            return Ok(String::from("Registro exitoso. Se te añadió en la cola de usuarios pendientes."));
        }

        /// Utilizado por un Administrador.
        /// Obtiene la información del próximo usuario a registrarse.
        #[ink(message)]
        pub fn obtener_informacion_siguiente_usuario_pendiente(&self) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            let sig_usuario = self.usuarios_pendientes.first();
            match sig_usuario {
                Some(usuario) => {
                    let mut str = String::from("Nombre: ") + usuario.nombre.as_str();
                    str.push_str((String::from("\nApellido: ") + usuario.apellido.as_str()).as_str());
                    str.push_str((String::from("\nDNI: ") + usuario.apellido.as_str()).as_str());
                    return Ok(str);
                },
                None => Err(String::from("No hay usuarios pendientes.")),
            }
        }

        /// Utilizado por un Administrador.
        /// Se procesará el próximo usuario pendiente.
        /// Para obtener la información del mismo, utilizar obtenerInformacionSiguienteUsuarioPendiente
        /// Si se acepta el usuario, podrá utilizar el sistema.
        /// Si se rechaza el usuario, este no podrá volver a intentar registrarse en el sistema.
        #[ink(message)]
        pub fn procesar_siguiente_usuario_pendiente(&mut self, aceptar_usuario:bool) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            let sig_usuario = self.usuarios_pendientes.first();
            if sig_usuario.is_none() { return Err(String::from("No hay usuarios pendientes.")); }

            let usuario = self.usuarios_pendientes.remove(0);
            if aceptar_usuario { 
                self.usuarios.push(usuario);
                return Ok(String::from("Usuario agregado exitosamente."));
            }

            self.usuarios_rechazados.push(usuario);
            return Ok(String::from("Usuario rechazado exitosamente."));
        }


       
        #[ink(message)]
        pub fn obtener_usuarios_pendientes(&self) -> Vec<AccountId>
        {
            self.usuarios_pendientes.iter().map(|usuario| usuario.id).collect()
        }
        #[ink(message)]
        pub fn obtener_usuarios_rechazados(&self) -> Vec<AccountId>
        {
            self.usuarios_rechazados.iter().map(|usuario| usuario.id).collect()
        }
        #[ink(message)]
        pub fn obtener_usuarios(&self) -> Vec<AccountId>
        {
            self.usuarios.iter().map(|usuario| usuario.id).collect()
        }

        #[ink(message)]
        pub fn obtener_datos_usuario_por_id(&mut self, id_usuario: AccountId) -> Result<String, String> 
        {
            let sig_usuario = self.obtener_usuario_por_id(id_usuario);
            match sig_usuario {
                Some(usuario) => {
                    let mut str = String::from("Nombre: ") + usuario.nombre.as_str();
                    str.push_str((String::from("\nApellido: ") + usuario.apellido.as_str()).as_str());
                    str.push_str((String::from("\nDNI: ") + usuario.apellido.as_str()).as_str());
                    return Ok(str);
                },
                None => Err(String::from("No hay usuarios con tal id.")),
            }
        }
        #[ink(message)]
        pub fn obtener_datos_usuario_pendiente_por_id(&mut self, id_usuario: AccountId) -> Result<String, String> 
        {
            let sig_usuario = self.obtener_usuario_por_id(id_usuario);
            match sig_usuario {
                Some(usuario) => {
                    let mut str = String::from("Nombre: ") + usuario.nombre.as_str();
                    str.push_str((String::from("\nApellido: ") + usuario.apellido.as_str()).as_str());
                    str.push_str((String::from("\nDNI: ") + usuario.apellido.as_str()).as_str());
                    return Ok(str);
                },
                None => Err(String::from("No hay usuarios con tal id.")),
            }
        }
        #[ink(message)]
        pub fn obtener_datos_usuario_rechazado_por_id(&mut self, id_usuario: AccountId) -> Result<String, String> 
        {
            let sig_usuario = self.obtener_usuario_por_id(id_usuario);
            match sig_usuario {
                Some(usuario) => {
                    let mut str = String::from("Nombre: ") + usuario.nombre.as_str();
                    str.push_str((String::from("\nApellido: ") + usuario.apellido.as_str()).as_str());
                    str.push_str((String::from("\nDNI: ") + usuario.apellido.as_str()).as_str());
                    return Ok(str);
                },
                None => Err(String::from("No hay usuarios con tal id.")),
            }
        }


        /// Utilizado por un administrador.
        /// Activa el registro de usuarios si no está activo el registro.
        #[ink(message)]
        pub fn activar_registro(&mut self) -> Result<String, String> 
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            if self.registro_activado { return Err(String::from("El registro ya está activado.")); }
            self.registro_activado = true;
            return Ok(String::from("Se activó el registro para los usuarios."));
        }
        /// Utilizado por un administrador.
        /// Desactiva el registro de usuarios si no está activo el registro.
        #[ink(message)]
        pub fn desactivar_registro(&mut self) -> Result<String, String> 
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            if !self.registro_activado { return Err(String::from("El registro ya está desactivado.")); }
            self.registro_activado = false;
            return Ok(String::from("Se desactivó el registro para los usuarios."));
        }


    // ===================================================================================================
    // ===================================================================================================
    // ===================================================================================================





        /// Utilizado por un Administrador.
        /// Se procesará el próximo usuario pendiente en una eleccion particular.
        /// y se lo coloca en el vector de candidato o votante en esa eleccion segun que quiera ser.
        pub fn procesar_usuarios_en_una_eleccion(&mut self, eleccion_id:u32,aceptar_usuario:bool) -> Result<String, String>
        {
                if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }

               let eleccion_elegida = match self.obtener_eleccion_por_id(eleccion_id) {
                Some(eleccion) => eleccion,
                None => return Err(String::from("Eleccion no encontrada")),
            };
            return eleccion_elegida.procesar_siguiente_usuario_pendiente(aceptar_usuario);

            
        }
    }
   
        


    /*#[cfg(test)]
    mod tests {
        use super::*;
    }*/
}
