#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[ink::contract]
mod TrabajoFinal {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use scale_info::prelude::format;
    use ink::prelude::string::ToString;

    enum ERRORES
    {
        NO_ES_ADMINISTRADOR,
        USUARIO_NO_REGISTRADO,
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
        id:u64,
        candidatos:Vec<CandidatoConteo>,
        votantes:Vec<Votante>,
        usuarios_rechazados:Vec<AccountId>,
        usuarios_pendientes:Vec<(AccountId,TIPO_DE_USUARIO)>,
        votacion_iniciada:bool,
        fecha_inicio:u64,
        fecha_final:u64,
    }

    impl Eleccion
    {
        fn contiene_usuario_pendiente(&self, id: AccountId) -> bool {
            self.usuarios_pendientes.iter().any(|(usuario_id, _tipo)| *usuario_id == id)
        }

        fn existe_candidato(&self, candidato_id:u32) -> bool
        {
            candidato_id >= 1 && candidato_id <= self.candidatos.len() as u32
        }

        fn obtener_informacion_candidato(&self, candidato_id:u32) -> Option<&CandidatoConteo> //Quizás se debería de cambiar a un campo específico del candidato como por ejemplo discurso/ideas, y también su nombre
        {
            if !self.existe_candidato(candidato_id) { return None; }
            match (candidato_id as usize).checked_sub(1) {
                None => None,
                Some(index) => Some(&self.candidatos[index])
            }
        }

        pub fn votar_candidato(&mut self, votante_id:AccountId, candidato_id:u32) -> Result<String, String>
        {
            if !self.existe_candidato(candidato_id) { return Err(String::from("No existe un candidato con este id.")); }

            let votante = match self.votantes.iter_mut().find(|votante| votante.id == votante_id) {
                Some(votante) => votante,
                None => return Err(String::from("No estás registrado en la elección."))
            };
            if votante.voto_emitido { return Err(String::from("No se realizó el voto porque ya votaste anteriormente.")); }
            votante.voto_emitido = true;

            let candidato = match (candidato_id as usize).checked_sub(1) {
                None => return Err(String::from("Se produjo un overflow intentando obtener el candidato.")),
                Some(index) => &mut self.candidatos[index]
            };
            match candidato.votos_totales.checked_add(1) {
                None => {
                    votante.voto_emitido = false;
                    return Err(String::from("Se produjo un overflow al intentar sumar el voto."));
                },
                Some(votos_totales) => {
                    candidato.votos_totales = votos_totales;
                    return Ok(String::from("Voto emitido exitosamente."));
                }
            }
        }

        ///Usado por el administrador.
        ///Revisa el primer usuario pendiente.
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

                    let candidato_id = match (self.candidatos.len() as u32).checked_add(1) {
                        Some(id_validado) => id_validado,
                        None => return Err(String::from("Ocurrio un overflow al calcular la ID del candidato.")),
                    };
                    self.candidatos.push(CandidatoConteo{
                        id:usuario,
                        candidato_id,
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
        usuarios_rechazados:Vec<AccountId>,
        elecciones:Vec<Eleccion>,
    }

    #[warn(clippy::arithmetic_side_effects)]
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

        fn obtener_usuario(&self, id:AccountId) -> Option<&Usuario> {
            let id = self.env().caller();
            self.usuarios.iter().find(|usuario| usuario.id == id)
        }

        fn es_usuario_registrado(&self) -> bool {
            let id = self.env().caller();
            self.usuarios.iter().any(|usuario| usuario.id == id)
        }

        fn es_usuario_pendiente(&self) -> bool
        {
            let id=self.env().caller();
            self.usuarios_pendientes.iter().any(|usuario| usuario.id == id)
        }

        fn existe_eleccion(&self, eleccion_id:u64) -> bool
        {
            if eleccion_id >= 1 && eleccion_id <= self.elecciones.len() as u64 {
                return true;
            }
            return false;
        }

        fn obtener_eleccion_por_id(&mut self, eleccion_id:u64) -> Option<&mut Eleccion>
        {
            if self.existe_eleccion(eleccion_id) {
                match eleccion_id.checked_sub(1) {
                    Some(index_valid) => return Some(&mut self.elecciones[index_valid as usize]),
                    None => return None
                }
            }
            return None;
        }

        fn obtener_ref_eleccion_por_id(&self, eleccion_id:u64) -> Option<&Eleccion>
        {
            if self.existe_eleccion(eleccion_id) {
                return Some(&self.elecciones[(eleccion_id - 1) as usize]);
            }
            return None;
        }

        fn es_administrador(&self) -> bool
        {
            self.env().caller() == self.administrador
        }


        fn validar_estado_eleccion(&mut self,eleccion_id:u64,block_timestamp:u64,id_usuario:AccountId) -> Result<&mut Eleccion,String>{
            let option_eleccion = self.obtener_eleccion_por_id(eleccion_id);
            if option_eleccion.is_none() { return Err(String::from("No existe una elección con ese id.")); }
            
            let eleccion = option_eleccion.unwrap();
            if eleccion.contiene_usuario_pendiente(id_usuario) { return Err(String::from("Ya está registrado en la elección.")); }
            
            if eleccion.votacion_iniciada || eleccion.fecha_inicio < block_timestamp {
                return Err(String::from("La votación en la elección ya comenzó, no te puedes registrar."));
            }
            if eleccion.fecha_final < block_timestamp {
                return Err(String::from("La elección ya finalizó, no te puedes registrar."));
            }
            Ok(eleccion)
        }

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

            let eleccion_id = match (self.elecciones.len() as u64).checked_add(1) {
                Some(index) => index,
                None => return Err(String::from("Se produjo un overflow al intentar crear una elección."))
            };
            let eleccion = Eleccion {
                id: eleccion_id,
                candidatos: Vec::new(),
                votantes: Vec::new(),
                usuarios_pendientes: Vec::new(),
                usuarios_rechazados: Vec::new(),
                votacion_iniciada:false,
                fecha_inicio: fecha_inicial_milisegundos.unwrap().and_utc().timestamp_millis() as u64,
                fecha_final: fecha_final_milisegundos.unwrap().and_utc().timestamp_millis() as u64,
            };
            self.elecciones.push(eleccion);

            return Ok(format!("Eleccion creada exitosamente. Id de la elección: {}", eleccion_id));
        }

        /// Utilizado por los usuarios registrados en el sistema para poder ingresar a una elección.
        /// Un usuario registrado y que no está registrado en la elección puede ingresar a la misma como candidato o votante.
        /// Estos no pueden ingresar a la misma si esta ya comenzó su periodo de votación o ya terminó la elección.
        /// Para ingresar como candidato es necesario una candidatura.
       
        
        #[ink(message)]
        pub fn ingresar_a_eleccion(&mut self, eleccion_id:u64, tipo:TIPO_DE_USUARIO) -> Result<String, String>
        {
            if !self.es_usuario_registrado() { return Err(ERRORES::USUARIO_NO_REGISTRADO.to_string()); }
            let id = self.env().caller();

            let block_timestamp = self.env().block_timestamp();
            let result = self.validar_estado_eleccion(eleccion_id, block_timestamp, id);
            let eleccion = match result {
                Ok(eleccion) => eleccion,
                Err(mensaje) => return Err(mensaje)
            };
            //Validar que un usuario que ya ha sido rechazado en la misma eleccion no intente volver a ponerse como pendiente 
            if eleccion.usuarios_rechazados.contains(&id) {
                return Err(String::from("Ya has sido rechazado no puedes ingresar a la eleccion"));
            }
            
            if eleccion.contiene_usuario_pendiente(id){
                return Err(String::from("No puedes ingresar dos veces a la misma eleccion"));
            }

            eleccion.usuarios_pendientes.push((id,tipo));

            return Ok(String::from("Ingresó a la elección correctamente Pendiente de aprobacion del Administrador"));
              
        }

        /// Utilizado por el administrador.
        /// El administrador puede iniciar una votación si esta no se inició cuando se alcanzó la fecha inicial de la misma.
        /// El administrador no puede iniciar si la fecha actual es menor a la fecha inicial establecida para la votación. 
        pub fn iniciar_votacion(&mut self, eleccion_id:u64) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            let block_timestamp = self.env().block_timestamp();

            match self.obtener_eleccion_por_id(eleccion_id) {
                Some(eleccion) => {
                    if block_timestamp > eleccion.fecha_final {
                        return Err(String::from("La votación ya finalizó."));
                    }
                    if eleccion.votacion_iniciada {
                        return Err(String::from("La votación ya inició."));
                    }
                    if block_timestamp < eleccion.fecha_inicio {
                        return Err(String::from("Todavía no es la fecha para la votación."));
                    }
                    eleccion.votacion_iniciada = true;
                    return Ok(String::from("Se inició la votación exitosamente."));
                },
                None => return Err(String::from("No existe una elección con ese id."))
            }
        }

        /// Utilizado por el administrador.
        /// Permite al administrador transferir el rol de administrador a otra persona.
        pub fn transferir_administrador(&mut self, id:AccountId) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            self.administrador = id;
            return Ok(String::from("Se transfirió el rol de administrador correctamente."));
        }

        /// Utilizado por los usuarios registrados en el sistema y que están en la elección como votantes.
        /// Si el usuario ya emitió su voto, no puede volver a votar en la misma elección.
        /// Si el usuario no es votante, no puede votar.
        /// Si el periodo de la votación no comenzó o terminó, no puede votar.
        #[ink(message)]
        pub fn votar_a_candidato(&mut self, eleccion_id:u64, candidato_id:u32) -> Result<String, String>
        {
            if !self.es_usuario_registrado() { return Err(ERRORES::USUARIO_NO_REGISTRADO.to_string()); }
            let id = self.env().caller();
            let block_timestamp = self.env().block_timestamp();

            match self.obtener_eleccion_por_id(eleccion_id) {
                Some(eleccion) => {
                    if !eleccion.votacion_iniciada {
                        if block_timestamp < eleccion.fecha_inicio {
                            return Err(String::from("Todavía no es la fecha para la votación."));
                        }
                        eleccion.votacion_iniciada = true;
                    }
                    if block_timestamp > eleccion.fecha_final {
                        return Err(String::from("La votación ya finalizó."));
                    }

                    return eleccion.votar_candidato(id, candidato_id);
                },
                None => return Err(String::from("No existe una elección con ese id."))
            }
        }

        /// Utilizado por los usuarios registrados en el sistema y que están en la elección ingresada.
        /// Se utiliza para poder obtener información de algún candidato en específico.
        /// Las IDs de los candidatos van de 1 a N.
        pub fn obtener_informacion_candidato(&self, eleccion_id:u64, candidato_id:u32) -> Result<String, String>
        {
            let eleccion_elegida = match self.obtener_ref_eleccion_por_id(eleccion_id) {
                Some(eleccion) => eleccion,
                None => return Err(String::from("No existe una elección con ese id."))
            };
            let candidato_elegido = match eleccion_elegida.obtener_informacion_candidato(candidato_id) {
                Some(candidato) => candidato,
                None => return Err(String::from("No existe una candidato con ese id."))
            };
            let usuario = self.obtener_usuario(candidato_elegido.id).unwrap();

            let mut informacion = String::from("Nombre: ");
            informacion.push_str(&usuario.nombre);
            informacion.push_str("\nApellido: ");
            informacion.push_str(&usuario.apellido);

            return Ok(informacion);
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

            self.usuarios_rechazados.push(usuario.id);
            return Ok(String::from("Usuario rechazado exitosamente."));
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

        /// Utilizado por los usuarios para poder registrarse en el sistema.
        /// Luego de registrarse queda pendiente de aceptación por parte de un Administrador.
        /// Si tu registro es rechazado, no podrás volver a intentar registrarte.
        #[ink(message)]
        pub fn registrarse(&mut self, nombre:String, apellido:String, dni:String) -> Result<String, String>
        {
            if !self.registro_activado { return Err(String::from("El registro todavía no está activado.")); }
            let id = self.env().caller();
            if self.es_administrador() { return Err(String::from("Eres el administrador, no puedes registrarte.")); }
            if self.usuarios_rechazados.contains(&id) { 
                return Err(String::from("Tu solicitud de registro ya fue rechazada."));
            }
            if self.es_usuario_registrado()
            {
                return Err(String::from("Ya estás registrado como usuario."));
            }
            if self.es_usuario_pendiente()
            {
                return Err(String::from("Ya estás en la cola de usuarios pendientes."));    
            }
            let usuario = Usuario { id, nombre, apellido, dni };
            self.usuarios_pendientes.push(usuario);
            return Ok(String::from("Registro exitoso. Se te añadió en la cola de usuarios pendientes."));
        }

         /// Utilizado por un Administrador.
        /// Se procesará el próximo usuario pendiente en una eleccion particular.
        /// y se lo coloca en el vector de candidato o votante en esa eleccion segun que quiera ser.
        pub fn procesar_usuarios_en_una_eleccion(&mut self, eleccion_id:u64,aceptar_usuario:bool) -> Result<String, String>
            {
                if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }

               let eleccion_elegida = match self.obtener_eleccion_por_id(eleccion_id) {
                Some(eleccion) => eleccion,
                None => return Err(String::from("Eleccion no encontrada")),
            };
            return eleccion_elegida.procesar_siguiente_usuario_pendiente(aceptar_usuario);
        }
    }
   
        
    

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test::default_accounts;
        use ink::env::DefaultEnvironment;
        

        #[test]
        fn test_no_es_administrador() {
            let error = ERRORES::NO_ES_ADMINISTRADOR;
            assert_eq!(error.to_string(), "No eres el administrador.");
        }
    
        #[test]
        fn test_usuario_no_registrado() {
            let error = ERRORES::USUARIO_NO_REGISTRADO;
            assert_eq!(error.to_string(), "No estás registrado en el sistema. Espera a que te acepten en el mismo o realiza la solicitud.");
        }


    
        fn setup_eleccion() -> Eleccion {
            Eleccion {
                id: 1,
                candidatos: Vec::new(),
                votantes: Vec::new(),
                usuarios_rechazados: Vec::new(),
                usuarios_pendientes: Vec::new(),
                votacion_iniciada: false,
                fecha_inicio: 0,
                fecha_final: 0,
            }
        }
    
        #[test]
        fn test_contiene_usuario_pendiente() {
            let mut eleccion = setup_eleccion();
            let accounts = default_accounts::<DefaultEnvironment>();//uentas predeterminadas utilizadas para tests
            
            eleccion.usuarios_pendientes.push((accounts.alice, TIPO_DE_USUARIO::VOTANTE));
            
            assert!(eleccion.contiene_usuario_pendiente(accounts.alice));
            assert!(!eleccion.contiene_usuario_pendiente(accounts.bob));
        }
    
        #[test]
        fn test_existe_candidato() {
            let mut eleccion = setup_eleccion();
            let accounts = default_accounts::<DefaultEnvironment>();
            
            eleccion.candidatos.push(CandidatoConteo {
                id: accounts.alice,
                candidato_id: 1,
                votos_totales: 0,
            });
    
            assert!(eleccion.existe_candidato(1));
            assert!(!eleccion.existe_candidato(2));
        }
    
        #[test]
        fn test_votar_candidato() {
            let mut eleccion = setup_eleccion();
            let accounts = default_accounts::<DefaultEnvironment>();
    
            eleccion.candidatos.push(CandidatoConteo {
                id: accounts.alice,
                candidato_id: 1,
                votos_totales: 0,
            });
    
            eleccion.votantes.push(Votante {
                id: accounts.bob,
                voto_emitido: false,
            });
    
            let result = eleccion.votar_candidato(accounts.bob, 1);
            assert_eq!(result, Ok(String::from("Voto emitido exitosamente.")));
            assert!(eleccion.votantes[0].voto_emitido);
            assert_eq!(eleccion.candidatos[0].votos_totales, 1);
        }
    
        #[test]
        fn test_procesar_siguiente_usuario_pendiente() {
            let mut eleccion = setup_eleccion();
            let accounts = default_accounts::<DefaultEnvironment>();
    
            eleccion.usuarios_pendientes.push((accounts.alice, TIPO_DE_USUARIO::VOTANTE));
            eleccion.usuarios_pendientes.push((accounts.bob, TIPO_DE_USUARIO::CANDIDATO));
    
            let result = eleccion.procesar_siguiente_usuario_pendiente(true);
            assert_eq!(result, Ok(String::from("Usuario agregado exitosamente.")));
            assert_eq!(eleccion.votantes.len(), 1);
            assert_eq!(eleccion.candidatos.len(), 0);
    
            let result = eleccion.procesar_siguiente_usuario_pendiente(true);
            assert_eq!(result, Ok(String::from("Usuario agregado exitosamente.")));
            assert_eq!(eleccion.votantes.len(), 1);
            assert_eq!(eleccion.candidatos.len(), 1);
    
            let result = eleccion.procesar_siguiente_usuario_pendiente(false);
            assert_eq!(result, Err(String::from("No hay usuarios pendientes.")));
        }

        #[test]
        fn test_obtener_informacion_candidato() {
            let mut eleccion = setup_eleccion();
            let accounts = default_accounts::<DefaultEnvironment>();
            eleccion.candidatos.push(CandidatoConteo {
                id: accounts.alice,
                candidato_id: 1,
                votos_totales: 0,
            });
            eleccion.candidatos.push(CandidatoConteo {
                id: accounts.bob,
                candidato_id: 2,
                votos_totales: 0,
            });

            let candidato_info = eleccion.obtener_informacion_candidato(1);
            assert!(candidato_info.is_some());
            assert_eq!(candidato_info.unwrap().id, accounts.alice);

            let candidato_info = eleccion.obtener_informacion_candidato(2);
            assert!(candidato_info.is_some());
            assert_eq!(candidato_info.unwrap().id, accounts.bob);
    
            let candidato_info = eleccion.obtener_informacion_candidato(3);
            assert!(candidato_info.is_none());
        }
        


    fn crear_usuario(id: AccountId, nombre: &str, apellido: &str, dni: &str) -> Usuario {
        Usuario {
            id,
            nombre: nombre.to_string(),
            apellido: apellido.to_string(),
            dni: dni.to_string(),
        }
    }
    fn crear_trabajo_final(administrador: AccountId) -> TrabajoFinal {
        TrabajoFinal {
            administrador,
            registro_activado: false,
            usuarios: Vec::new(),
            usuarios_pendientes: Vec::new(),
            usuarios_rechazados: Vec::new(),
            elecciones: Vec::new(),
        }
    }

    #[ink::test]
    fn test_obtener_usuario() {
        let accounts = default_accounts::<DefaultEnvironment>();
        let id: AccountId = accounts.alice;
        let mut trabajo_final = TrabajoFinal::new();
        let usuario = crear_usuario(id, "Juan", "Perez", "12345678");
        trabajo_final.usuarios.push(usuario);

        let result = trabajo_final.obtener_usuario(id);
        assert!(result.is_some());
        let user = result.unwrap();
        assert_eq!(user.nombre, "Juan");
        assert_eq!(user.apellido, "Perez");
        assert_eq!(user.dni, "12345678");
    }

    #[ink::test]
    fn test_es_usuario_registrado() {
        let accounts = default_accounts::<DefaultEnvironment>();
        let id: AccountId = accounts.alice;
        let mut trabajo_final = TrabajoFinal::new();
        let usuario = crear_usuario(id, "Juan", "Perez", "12345678");
        trabajo_final.usuarios.push(usuario);

        assert!(trabajo_final.es_usuario_registrado());
    }

    #[ink::test]
    fn test_es_usuario_pendiente() {
        let accounts = default_accounts::<DefaultEnvironment>();
        let id: AccountId = accounts.alice;
        let mut trabajo_final = TrabajoFinal::new();
        let usuario = crear_usuario(id, "Juan", "Perez", "12345678");
        trabajo_final.usuarios_pendientes.push(usuario);

        assert!(trabajo_final.es_usuario_pendiente());
    }

    #[test]
    fn test_existe_eleccion() {
        let id: AccountId = [0; 32].into();
        let mut trabajo_final = crear_trabajo_final(id);
        let eleccion = setup_eleccion();
        trabajo_final.elecciones.push(eleccion);

        assert!(trabajo_final.existe_eleccion(1));
        assert!(!trabajo_final.existe_eleccion(2));
    }

    #[test]
    fn test_obtener_eleccion_por_id() {
        let id: AccountId = [0; 32].into();
        let mut trabajo_final = crear_trabajo_final(id);
        let eleccion = setup_eleccion();
        trabajo_final.elecciones.push(eleccion);

        let result = trabajo_final.obtener_eleccion_por_id(1);
        assert!(result.is_some());
        let eleccion_obtenida = result.unwrap();
        assert_eq!(eleccion_obtenida.id, 1);

        assert!(trabajo_final.obtener_eleccion_por_id(2).is_none());
    }

    }

   
}



