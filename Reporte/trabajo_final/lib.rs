#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::TrabajoFinal::{TrabajoFinal as StructTrabajoFinal, TrabajoFinalRef};

#[ink::contract]
mod TrabajoFinal {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use scale_info::prelude::format;
    use ink::env::test::advance_block;
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

    #[derive(scale::Decode, scale::Encode, Debug,Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    struct Usuario
    {
        id:AccountId,
        nombre:String,
        apellido:String,
        dni:String,
    }

    #[derive(scale::Decode, scale::Encode, Debug,Clone,PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Votante
    {
        id:AccountId,
        voto_emitido:bool,
    }

    #[derive(scale::Decode, scale::Encode, Debug,Clone,PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    struct CandidatoConteo
    {
        id:AccountId,
        candidato_id:u32,
        votos_totales:u32,
    }

    #[derive(scale::Decode, scale::Encode, Debug,PartialEq)]
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
        resultados:Option<Resultados>
    }

    #[derive(scale::Decode, scale::Encode, Debug, Clone,PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Resultados
    {
        votos_totales:u64, // Votos totales, cuentan los que votaron y no votaron
        votos_realizados:u64, // Votos realizados, cuentan solo los que votaron
        votos_candidatos:Vec<(AccountId, u64)>
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

        fn obtener_resultados_votacion(&mut self, block_timestamp:u64) -> Option<&Resultados>
        {
            if self.fecha_final > block_timestamp {
                return None;
            }

            if self.resultados.is_some() {
                return self.resultados.as_ref();
            }

            let mut resultados = Resultados { 
                votos_totales: 0, 
                votos_realizados: 0,
                votos_candidatos: Vec::new(),
            };

            resultados.votos_totales = self.votantes.len() as u64;
            resultados.votos_realizados = self.votantes.iter().filter(|v| v.voto_emitido).count() as u64;
            self.candidatos.iter().for_each(|c| {
                resultados.votos_candidatos.push((c.id, c.votos_totales as u64));
            });

            self.resultados = Some(resultados);
            return self.resultados.as_ref();
        }
    }

    #[ink(storage)]
    pub struct TrabajoFinal {
        administrador:AccountId,
        generador_reportes:Option<AccountId>,
        registro_activado:bool,
        usuarios:Vec<Usuario>,
        usuarios_pendientes:Vec<Usuario>,
        usuarios_rechazados:Vec<AccountId>,
        elecciones:Vec<Eleccion>,
    }

    #[ink(impl)]
    impl TrabajoFinal {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { 
                administrador: Self::env().caller(),
                generador_reportes: None,
                registro_activado: false,
                usuarios: Vec::new(),
                usuarios_pendientes: Vec::new(),
                usuarios_rechazados: Vec::new(),
                elecciones: Vec::new(),
            }
        }
        
        fn es_generador_reportes(&self) -> bool
        {
            match self.generador_reportes { 
                None => false,
                Some(val) => self.env().caller() == val
            }
        }

        fn es_administrador(&self) -> bool
        {
            self.env().caller() == self.administrador
        }

        fn obtener_usuario(&self, id: AccountId) -> Option<&Usuario> {
            // let id = self.env().caller();
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
                 match eleccion_id.checked_sub(1) {
                     Some(index_valid) => return Some(&self.elecciones[index_valid as usize]),
                     None => return None
                 }
             }
             return None;
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

        //  ----- Inicio Metodos publicos -------
        //  ----- Inicio Metodos publicos -------
        //  ----- Inicio Metodos publicos -------

        /// Utilizado por los usuarios para poder registrarse en el sistema.
        /// Luego de registrarse queda pendiente de aceptación por parte de un Administrador.
        /// Si tu registro es rechazado, no podrás volver a intentar registrarte.
        #[ink(message)]
        pub fn registrarse(&mut self, nombre:String, apellido:String, dni:String) -> Result<String, String>
        {
            self.registrarse_privado(nombre, apellido, dni)
        }
        fn registrarse_privado(&mut self, nombre:String, apellido:String, dni:String) -> Result<String, String>
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
        /// Obtiene la información del próximo usuario a registrarse.
        #[ink(message)]
        pub fn obtener_informacion_siguiente_usuario_pendiente(&self) -> Result<String, String>
        {
            self.obtener_informacion_siguiente_usuario_pendiente_privado()
        }
        fn obtener_informacion_siguiente_usuario_pendiente_privado(&self) -> Result<String, String>
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
            self.procesar_siguiente_usuario_pendiente_privado(aceptar_usuario)
        }
        fn procesar_siguiente_usuario_pendiente_privado(&mut self, aceptar_usuario:bool) -> Result<String, String>
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


        //  ----- METODOS ELECCIONES -------
        //  ----- METODOS ELECCIONES -------
        //  ----- METODOS ELECCIONES -------

        /// Utilizado por un administrador.
        /// Crea una elección colocando fecha de inicio y final (Las fechas para que se correspondan a nuestro horario GTM-3/UTC-3 hay que sumarle 3 a la hora).
        #[ink(message)]
        pub fn crear_eleccion(&mut self, fecha_inicial:String, fecha_final:String) -> Result<String, String>
        {
            self.crear_eleccion_privado(fecha_inicial, fecha_final)
        }
        fn crear_eleccion_privado(&mut self, fecha_inicial: String, fecha_final: String) -> Result<String, String> 
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
    
            let fecha_inicial_milisegundos = chrono::NaiveDateTime::parse_from_str(&fecha_inicial, "%d-%m-%Y %H:%M");
            if fecha_inicial_milisegundos.is_err() {
                return Err(String::from("Error en el formato de la fecha inicial. Formato: dd-mm-YYYY hh:mm"));
            }
            let fecha_final_milisegundos = chrono::NaiveDateTime::parse_from_str(&fecha_final, "%d-%m-%Y %H:%M");
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
                resultados:None
            };
            self.elecciones.push(eleccion);
    
            return Ok(format!("Eleccion creada exitosamente. Id de la elección: {}", eleccion_id));
        }

                
        /// Utilizado por el administrador.
        /// El administrador puede iniciar una votación si esta no se inició cuando se alcanzó la fecha inicial de la misma.
        /// El administrador no puede iniciar si la fecha actual es menor a la fecha inicial establecida para la votación. 
        #[ink(message)]
        pub fn iniciar_votacion(&mut self, eleccion_id:u64) -> Result<String, String>
        {
            self.iniciar_votacion_privado(eleccion_id)
        }

        pub fn iniciar_votacion_privado(&mut self, eleccion_id:u64) -> Result<String, String>
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

        /// Utilizado por los usuarios registrados en el sistema para poder ingresar a una elección.
        /// Un usuario registrado y que no está registrado en la elección puede ingresar a la misma como candidato o votante.
        /// Estos no pueden ingresar a la misma si esta ya comenzó su periodo de votación o ya terminó la elección.
        /// Para ingresar como candidato es necesario una candidatura.   
        #[ink(message)]
        pub fn ingresar_a_eleccion(&mut self, eleccion_id:u64, tipo:TIPO_DE_USUARIO) -> Result<String, String>
        {
            self.ingresar_a_eleccion_privado(eleccion_id,tipo)
        }
        fn ingresar_a_eleccion_privado(&mut self, eleccion_id:u64, tipo:TIPO_DE_USUARIO) -> Result<String, String>
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

        /// Utilizado por un Administrador.
        /// Obtiene la información del próximo usuario a registrarse.
        #[ink(message)]
        pub fn obtener_siguiente_usuario_pendiente_en_una_eleccion(&mut self, eleccion_id:u64) -> Result<String, String>
        {
            self.obtener_siguiente_usuario_pendiente_en_una_eleccion_privado(eleccion_id)
        }
        pub fn obtener_siguiente_usuario_pendiente_en_una_eleccion_privado(&mut self, eleccion_id:u64) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            let eleccion_elegida = match self.obtener_eleccion_por_id(eleccion_id) {
                Some(eleccion) => eleccion,
                None => return Err(String::from("Eleccion no encontrada")),
            };
            let sig_usuario = eleccion_elegida.usuarios_pendientes.first();
            match sig_usuario {
                Some(usuario_eleccion) => {
                    let mut datos_usuario = String::from("Usuario: ");
                    datos_usuario.push_str( hex::encode(usuario_eleccion.0).as_str() );
                    match usuario_eleccion.1 {
                        TIPO_DE_USUARIO::VOTANTE => datos_usuario.push_str("\nEl usuario quiere ser un VOTANTE"),
                        TIPO_DE_USUARIO::CANDIDATO => datos_usuario.push_str("\nEl usuario quiere ser un CANDIDATO")
                    };
                    Ok(datos_usuario)
                },
                None => Err(String::from("No hay usuarios pendientes.")),
            }
        }


        /// Utilizado por un Administrador.
        /// Se procesará el próximo usuario pendiente en una eleccion particular.
        /// y se lo coloca en el vector de candidato o votante en esa eleccion segun que quiera ser.
        #[ink(message)]
        pub fn procesar_usuarios_en_una_eleccion(&mut self, eleccion_id:u64,aceptar_usuario:bool) -> Result<String, String>
        {
            self.procesar_usuarios_en_una_eleccion_privado(eleccion_id,aceptar_usuario)
        }
        pub fn procesar_usuarios_en_una_eleccion_privado(&mut self, eleccion_id:u64,aceptar_usuario:bool) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            
            let eleccion_elegida = match self.obtener_eleccion_por_id(eleccion_id) {
                Some(eleccion) => eleccion,
                None => return Err(String::from("Eleccion no encontrada")),
            };
            return eleccion_elegida.procesar_siguiente_usuario_pendiente(aceptar_usuario);
        }
        
        /// Utilizado por los usuarios registrados en el sistema y que están en la elección como votantes.
        /// Si el usuario ya emitió su voto, no puede volver a votar en la misma elección.
        /// Si el usuario no es votante, no puede votar.
        /// Si el periodo de la votación no comenzó o terminó, no puede votar.
        #[ink(message)]
        pub fn votar_a_candidato(&mut self, eleccion_id:u64, candidato_id:u32) -> Result<String, String>
        {
            self.votar_a_candidato_privado(eleccion_id, candidato_id)
        }
        fn votar_a_candidato_privado(&mut self, eleccion_id:u64, candidato_id:u32) -> Result<String, String>
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

        // ====-----==== METODOS USADOS POR EL ADMINISTRADOR ====----====
        // ====-----==== METODOS USADOS POR EL ADMINISTRADOR ====----====
        // ====-----==== METODOS USADOS POR EL ADMINISTRADOR ====----====

        /// Utilizado por un administrador.
        /// Activa el registro de usuarios si no está activo el registro.
        #[ink(message)]
        pub fn activar_registro(&mut self) -> Result<String, String>
        {
            self.activar_registro_privado()
        }
        fn activar_registro_privado(&mut self) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            if self.registro_activado { return Err(String::from("El registro ya está activado.")); }
            self.registro_activado = true;
            return Ok(String::from("Se activó el registro para los usuarios."));
        }

        /// Utilizado por un administrador.
        /// desActiva el registro de usuarios si no está activo el registro.
        #[ink(message)]
        pub fn desactivar_registro(&mut self) -> Result<String, String>
        {
            self.desactivar_registro_privado()
        }
        fn desactivar_registro_privado(&mut self) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            if !self.registro_activado { return Err(String::from("El registro ya está desactivado.")); }
            self.registro_activado = false;
            return Ok(String::from("Se desactivó el registro para los usuarios."));
        }

        /// Utilizado por el administrador.
        /// Permite al administrador transferir el rol de administrador a otra persona.
        #[ink(message)]
        pub fn transferir_administrador(&mut self, id:AccountId) -> Result<String, String>
        {
            self.transferir_administrador_privado(id)
        }
        pub fn transferir_administrador_privado(&mut self, id:AccountId) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            self.administrador = id;
            return Ok(String::from("Se transfirió el rol de administrador correctamente."));
        }
        
        /// Utilizado por el administrador.
        #[ink(message)]
        pub fn asignar_generador_reportes(&mut self, id:AccountId) -> Result<String, String>
        {
            self.asignar_generador_reportes_privado(id)
        }
        pub fn asignar_generador_reportes_privado(&mut self, id:AccountId) -> Result<String, String>
        {
            if !self.es_administrador() { return Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()); }
            self.generador_reportes = Some(id);
            return Ok(String::from("Se asigno el generador reportes correctamente."));
        }


        // ====-----==== METODOS PARA QUE USE EL OTRO CONTRATO ====----====
        // ====-----==== METODOS PARA QUE USE EL OTRO CONTRATO ====----====
        // ====-----==== METODOS PARA QUE USE EL OTRO CONTRATO ====----====

        #[ink(message)]
        pub fn obtener_informacion_usuario(&self, user_id: AccountId) -> Option<(String, String, String)> 
        {
            self.obtener_informacion_usuario_privado(user_id)
        }
        pub fn obtener_informacion_usuario_privado(&self, user_id: AccountId) -> Option<(String, String, String)> 
        {
            if !self.es_generador_reportes() { return None; }

            let option_usuario = self.usuarios.iter().find(|usuario| usuario.id == user_id);
            match option_usuario {
                None => None,
                Some(usuario) => Some( (usuario.nombre.clone(),  usuario.apellido.clone(),  usuario.dni.clone()) )
            }
        }

        #[ink(message)]
        pub fn obtener_votantes_eleccion_por_id(&mut self, eleccion_id: u64) -> Result<Vec<(AccountId,bool)>, String>
        {
            self.obtener_votantes_eleccion_por_id_privado(eleccion_id)
        }
        pub fn obtener_votantes_eleccion_por_id_privado(&mut self, eleccion_id: u64) -> Result<Vec<(AccountId,bool)>, String>
        {
            if !self.es_generador_reportes() { return Err(String::from("No es el generador de reportes!")); }
            let block_timestamp = self.env().block_timestamp();
            
            let eleccion_option = self.obtener_eleccion_por_id(eleccion_id);
            match eleccion_option {
                Some(eleccion) => {
                    // verificacion pobre
                    if eleccion.fecha_final > block_timestamp {
                        return Err(String::from("La elección no finalizó, no puedes obtener los datos."));
                    }
                    
                    Ok(eleccion.votantes.iter().map(|votante| (votante.id, votante.voto_emitido)).collect())
                },
                None => Err(String::from("La eleccion enviada no existe!")),
            }
        }
        #[ink(message)]
        pub fn obtener_candidatos_eleccion_por_id(&mut self, eleccion_id: u64) -> Result<Vec<(AccountId,u32)>, String>
        {
            self.obtener_candidatos_eleccion_por_id_privado(eleccion_id)
        }
        pub fn obtener_candidatos_eleccion_por_id_privado(&mut self, eleccion_id: u64) -> Result<Vec<(AccountId,u32)>, String>
        {
            if !self.es_generador_reportes() { return Err(String::from("No es el generador de reportes!")); }
            let block_timestamp = self.env().block_timestamp();

            match self.obtener_eleccion_por_id(eleccion_id){
                Some(eleccion) => {
                    // verificacion pobre
                    if eleccion.fecha_final > block_timestamp {
                        return Err(String::from("La elección no finalizó, no puedes obtener los datos."));
                    }
                    
                    Ok(eleccion.candidatos.iter().map(|candidato| (candidato.id, candidato.votos_totales)).collect())
                },
                None => Err(String::from("La eleccion enviada no existe!")),
            }
        }

        fn obtener_resultados_privado(&mut self,eleccion_id: u64) -> Result<Resultados, String> {
            let block_timestamp= self.env().block_timestamp();
            let eleccion = match self.obtener_eleccion_por_id(eleccion_id){
                Some(eleccion) => eleccion,
                None => return Err(String::from("No se encontró una elección con ese id."))
            };
    
            match eleccion.obtener_resultados_votacion(block_timestamp) {
                None => Err(String::from("Todavía no están los resultados de la elección publicados.")),
                Some(resultados) => Ok(resultados.clone())
            }
        }
        #[ink(message)]
        pub fn obtener_resultados(&mut self, eleccion_id:u64) -> Result<Resultados, String> {
            match self.obtener_resultados_privado(eleccion_id){
                Err(mensaje) => Err(mensaje),
                Ok(mensaje) =>  Ok(mensaje)
            }
        }
    }
   
        
    

    #[cfg(test)]
    mod tests {
        use core::ops::{AddAssign, SubAssign};

        use super::*;
        use ink::codegen::Env;
        use ink::env::test::advance_block;
        use ink::env::test::{
            default_accounts, get_account_balance, recorded_events,
            DefaultAccounts, EmittedEvent
        };
        use ink::env::DefaultEnvironment;
        
        fn get_default_test_accounts(
        ) -> DefaultAccounts<ink::env::DefaultEnvironment> {
            default_accounts::<ink::env::DefaultEnvironment>()
        }
    
        // Sets caller returned by the next `Self::env().caller()` method call
        // in the contract.
        fn set_caller(caller: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(caller);
        }
    
            // test errores
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
            // test errores

        
            #[test]
            fn test_error_usuario_no_registrado() {
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
                resultados:None
            }
        }
    
    
        // ====================== INICIO TESTS ELECCION ======================
        // ====================== INICIO TESTS ELECCION ======================
        // ====================== INICIO TESTS ELECCION ======================
        #[test]
        fn test_contiene_usuario_pendiente() {
            let mut eleccion = setup_eleccion();
            let accounts = get_default_test_accounts();//uentas predeterminadas utilizadas para tests
            
            eleccion.usuarios_pendientes.push((accounts.alice, TIPO_DE_USUARIO::VOTANTE));
            
            assert!(eleccion.contiene_usuario_pendiente(accounts.alice));
            assert!(!eleccion.contiene_usuario_pendiente(accounts.bob));
        }
    
        #[test]
        fn test_existe_candidato() {
            let mut eleccion = setup_eleccion();
            let accounts = get_default_test_accounts();
            
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
            let accounts = get_default_test_accounts();
    
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
            let accounts = get_default_test_accounts();
    
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
            let accounts = get_default_test_accounts();
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

        #[test]
        fn test_procesar_siguiente_usuario_pendiente_eleccion() {
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
        // ====================== FIN TESTS ELECCION ======================
        // ====================== FIN TESTS ELECCION ======================
        // ====================== FIN TESTS ELECCION ======================
    
    
        
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
                generador_reportes: None,
                registro_activado: false,
                usuarios: Vec::new(),
                usuarios_pendientes: Vec::new(),
                usuarios_rechazados: Vec::new(),
                elecciones: Vec::new(),
            }
        }
        
        // ====================== INICIO TESTS SISTEMA ELECCIONES ======================
        // ====================== INICIO TESTS SISTEMA ELECCIONES ======================
        // ====================== INICIO TESTS SISTEMA ELECCIONES ======================
        
        #[test]
        fn test_constructor() {
            let accounts = get_default_test_accounts();
            let alice = accounts.alice;
            let charlie = accounts.charlie;
            set_caller(alice);
    
            let sistema_elecciones = TrabajoFinal::new();
            assert_eq!(sistema_elecciones.registro_activado, false);
            assert_eq!(sistema_elecciones.administrador, alice);
            assert_ne!(sistema_elecciones.administrador, charlie);
        }
        #[ink::test]
        fn test_obtener_informacion_candidato_eleccion() {
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
        #[test]
        fn test_obtener_informacion_siguiente_usuario_pendiente() {
            let administrador: AccountId = AccountId::from([0x1; 32]);
            let otro_usuario: AccountId = AccountId::from([0x2; 32]);
            set_caller(administrador);

            let mut contrato = TrabajoFinal::new();
            
            let usuario = Usuario { id: (otro_usuario), nombre: ("Joaquin".to_string()), apellido: ("Fontana".to_string()), dni: ("22222222".to_string()) };
            let mut str = String::from("Nombre: ") + usuario.nombre.as_str();
            str.push_str((String::from("\nApellido: ") + usuario.apellido.as_str()).as_str());
            str.push_str((String::from("\nDNI: ") + usuario.apellido.as_str()).as_str());
            //Intentar obtener informacion sin usuarios pendientes
            let result = contrato.obtener_informacion_siguiente_usuario_pendiente();
            assert!(result.is_err());
            
            contrato.usuarios_pendientes.push(usuario);

            let result = contrato.obtener_informacion_siguiente_usuario_pendiente();
            assert!(result.is_ok_and(|info| info == str));
        }
        #[test]
        fn test_transferir_administrador() {
            let accounts = get_default_test_accounts();
            let alice = accounts.alice;
            let charlie = accounts.charlie;
            let bob = accounts.bob;
            set_caller(alice);
    
            let mut sistema_elecciones = TrabajoFinal::new();
    
            let result = sistema_elecciones.transferir_administrador_privado(charlie);
            assert!(result.is_ok());
            assert_ne!(sistema_elecciones.administrador, alice);
            assert_eq!(sistema_elecciones.administrador, charlie);
    
            set_caller(bob);
            let result = sistema_elecciones.transferir_administrador_privado(charlie);
            assert!(result.is_err());
            assert_ne!(sistema_elecciones.administrador, alice);
            assert_eq!(sistema_elecciones.administrador, charlie);
        }
    
        #[ink::test]
        fn test_obtener_usuario() {
            let accounts = get_default_test_accounts();
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
            let accounts = get_default_test_accounts();
            let id: AccountId = accounts.alice;
            let mut trabajo_final = TrabajoFinal::new();
            let usuario = crear_usuario(id, "Juan", "Perez", "12345678");
            trabajo_final.usuarios.push(usuario);
    
            assert!(trabajo_final.es_usuario_registrado());
        }
    
        #[ink::test]
        fn test_es_usuario_pendiente() {
            let accounts = get_default_test_accounts();
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


        #[ink::test]
        fn test_activar_registro() {
            let mut contract = TrabajoFinal::new();

            let res = contract.activar_registro_privado();
            assert_eq!(res, Ok(String::from("Se activó el registro para los usuarios.")));
            assert_eq!(contract.registro_activado, true);

            let res = contract.activar_registro_privado();
            assert_eq!(res, Err(String::from("El registro ya está activado.")));
        }

        #[ink::test]
        fn test_registro_usuario() {
            let administrador: AccountId = AccountId::from([0x1; 32]);
            let otro_usuario: AccountId = AccountId::from([0x2; 32]);

            set_caller(administrador);
            let mut contrato = TrabajoFinal::new();

            contrato.activar_registro_privado().unwrap();

            set_caller(otro_usuario);

            let resultado = contrato.registrarse_privado("John".to_string(), "Doe".to_string(), "12345678".to_string());
            assert_eq!(resultado, Ok("Registro exitoso. Se te añadió en la cola de usuarios pendientes.".to_string()));
        }

        #[ink::test]
        fn test_obtener_ref_eleccion_por_id() {
            let mut contrato = TrabajoFinal::new();

            let candidato1 = CandidatoConteo {
                id: AccountId::from([0x01; 32]),
                candidato_id: 1,
                votos_totales: 0,
            };
            let candidato2 = CandidatoConteo {
                id: AccountId::from([0x02; 32]),
                candidato_id: 2,
                votos_totales: 0,
            };

            let votante1 = Votante {
                id: AccountId::from([0x03; 32]),
                voto_emitido: false,
            };
            let votante2 = Votante {
                id: AccountId::from([0x04; 32]),
                voto_emitido: false,
            };

            contrato.elecciones.push(Eleccion {
                id: 1,
                candidatos: vec![candidato1],
                votantes: vec![votante1],
                usuarios_rechazados: vec![],
                usuarios_pendientes: vec![],
                votacion_iniciada: false,
                fecha_inicio: 0,
                fecha_final: 0,
                resultados:None
            });

            contrato.elecciones.push(Eleccion {
                id: 2,
                candidatos: vec![candidato2],
                votantes: vec![votante2],
                usuarios_rechazados: vec![],
                usuarios_pendientes: vec![],
                votacion_iniciada: false,
                fecha_inicio: 0,
                fecha_final: 0,
                resultados:None
            });

            let resultado = contrato.obtener_ref_eleccion_por_id(1);
            assert!(resultado.is_some());
            assert_eq!(resultado.unwrap(), &contrato.elecciones[0]);

            let resultado = contrato.obtener_ref_eleccion_por_id(2);
            assert!(resultado.is_some());
            assert_eq!(resultado.unwrap(), &contrato.elecciones[1]);

            let resultado = contrato.obtener_ref_eleccion_por_id(3);
            assert!(resultado.is_none());
        }

        #[ink::test]
        fn test_validar_estado_eleccion() {
            let mut contrato = TrabajoFinal::new();
            let usuario_id = AccountId::from([0x01; 32]);
            
            contrato.elecciones.push(Eleccion {
                id: 1,
                candidatos: vec![],
                votantes: vec![],
                usuarios_rechazados: vec![],
                usuarios_pendientes: vec![(usuario_id, TIPO_DE_USUARIO::VOTANTE)],
                votacion_iniciada: false,
                fecha_inicio: 100,
                fecha_final: 200,
                resultados:None
            });

            contrato.elecciones.push(Eleccion {
                id: 2,
                candidatos: vec![],
                votantes: vec![],
                usuarios_rechazados: vec![],
                usuarios_pendientes: vec![],
                votacion_iniciada: true,
                fecha_inicio: 100,
                fecha_final: 200,
                resultados:None
            });

            contrato.elecciones.push(Eleccion {
                id: 3,
                candidatos: vec![],
                votantes: vec![],
                usuarios_rechazados: vec![],
                usuarios_pendientes: vec![],
                votacion_iniciada: false,
                fecha_inicio: 100,
                fecha_final: 50,
                resultados:None
            });

            contrato.elecciones.push(Eleccion {
                id: 4,
                candidatos: vec![],
                votantes: vec![],
                usuarios_rechazados: vec![],
                usuarios_pendientes: vec![],
                votacion_iniciada: false,
                fecha_inicio: 150,
                fecha_final: 200,
                resultados:None
            });

            // Caso 1: Usuario ya registrado
            let resultado = contrato.validar_estado_eleccion(1, 50, usuario_id);
            assert_eq!(resultado, Err(String::from("Ya está registrado en la elección.")));

            // Caso 2: Votación ya comenzó
            let resultado = contrato.validar_estado_eleccion(2, 50, usuario_id);
            assert_eq!(resultado, Err(String::from("La votación en la elección ya comenzó, no te puedes registrar.")));

            // Caso 3: Elección ya finalizó
            let resultado = contrato.validar_estado_eleccion(3, 52, usuario_id);
            assert_eq!(resultado, Err(String::from("La elección ya finalizó, no te puedes registrar.")));

            // Caso 4: Elección válida y no iniciada
            let resultado = contrato.validar_estado_eleccion(4, 50, usuario_id);
            assert!(resultado.is_ok());
            let eleccion = resultado.unwrap();
            assert_eq!(eleccion.id, 4);
        }

        #[ink::test]
        fn test_crear_eleccion() {
            // Instanciar el contrato
            let mut contrato = TrabajoFinal::new();
    
            // Configurar el administrador
            let administrador = AccountId::from([0x1; 32]);
            contrato.administrador = administrador;
    
            // Crear una elección válida
            let resultado = contrato.crear_eleccion_privado(
                "01-01-2025 12:00".to_string(),
                "31-01-2025 12:00".to_string()
            );
    
            // Verificar que la elección se creó correctamente
            assert_eq!(resultado, Ok("Eleccion creada exitosamente. Id de la elección: 1".to_string()));
    
            // Verificar que la elección se añadió a la lista
            assert_eq!(contrato.elecciones.len(), 1);
            let eleccion = &contrato.elecciones[0];
            assert_eq!(eleccion.id, 1);
            assert_eq!(eleccion.fecha_inicio, 1735732800000); 
            assert_eq!(eleccion.fecha_final, 1738324800000);
    
            // Crear una elección con fecha inicial inválida
            let resultado = contrato.crear_eleccion_privado(
                "01-01-2025 12:00".to_string(),
                "invalid-date".to_string()
            );
            assert_eq!(resultado, Err("Error en el formato de la fecha final. Formato: dd-mm-YYYY hh:mm".to_string()));
    
            // Crear una elección con fecha final inválida
            let resultado = contrato.crear_eleccion_privado(
                "invalid-date".to_string(),
                "31-01-2025 12:00".to_string()
            );
            assert_eq!(resultado, Err("Error en el formato de la fecha inicial. Formato: dd-mm-YYYY hh:mm".to_string()));
    
            // Crear una elección sin ser administrador
            contrato.administrador = AccountId::from([0x2; 32]);
            let resultado = contrato.crear_eleccion_privado(
                "01-01-2025 12:00".to_string(),
                "31-01-2025 12:00".to_string()
            );
            assert_eq!(resultado, Err(ERRORES::NO_ES_ADMINISTRADOR.to_string()));
        }

        #[ink::test]
        fn test_procesar_usuarios_en_una_eleccion() {
            let mut contrato = TrabajoFinal::new();
            let accounts = default_accounts::<DefaultEnvironment>();
            
            contrato.crear_eleccion_privado(
                String::from("01-07-2024 12:00"),
                String::from("02-07-2024 12:00"),
            ).unwrap();

            
            contrato.elecciones[0].usuarios_pendientes.push((accounts.alice, TIPO_DE_USUARIO::VOTANTE));
            contrato.elecciones[0].usuarios_pendientes.push((accounts.bob, TIPO_DE_USUARIO::CANDIDATO));

    
            // Caso 1: Procesar siguiente usuario aceptando
            let result = contrato.procesar_usuarios_en_una_eleccion(1, true);
            assert_eq!(result, Ok(String::from("Usuario agregado exitosamente.")));
    
            // Caso 2: Procesar siguiente usuario aceptando como candidato
            let result = contrato.procesar_usuarios_en_una_eleccion(1, true);
            assert_eq!(result, Ok(String::from("Usuario agregado exitosamente.")));
    
            // Caso 3: Procesar siguiente usuario rechazando
            let result = contrato.procesar_usuarios_en_una_eleccion(1, false);
            assert_eq!(result, Err(String::from("No hay usuarios pendientes.")));
    
            // Caso 4: Intentar procesar usuario en una elección no existente
            let result = contrato.procesar_usuarios_en_una_eleccion(2, true);
            assert_eq!(result, Err(String::from("Eleccion no encontrada")));
        } 


        #[ink::test]
        fn test_ingresar_a_eleccion2() {
            let accounts = get_default_test_accounts();
            let alice = accounts.alice;
            let charlie = accounts.charlie;
            let eleccion_id: u64 = 1;
            let tipo_usuario: TIPO_DE_USUARIO = TIPO_DE_USUARIO::VOTANTE;
            set_caller(alice);
        
            let mut contract = TrabajoFinal::new();
        
            // Establecemos el administrador como el llamante y activamos el registro
            contract.activar_registro().unwrap();
            contract.crear_eleccion("01-01-2024 10:00".into(), "02-01-2024 10:00".into()).unwrap();
        
            // Usuario no registrado intenta ingresar a la elección
            set_caller(charlie);
            let result = contract.ingresar_a_eleccion_privado(eleccion_id, tipo_usuario.clone());
            assert_eq!(result, Err(ERRORES::USUARIO_NO_REGISTRADO.to_string()), "Error: Usuario no registrado");
        
            // Registramos al usuario
            let result = contract.registrarse("Juan".into(), "Perez".into(), "12345678".into()).unwrap();
            assert_eq!(result, String::from("Registro exitoso. Se te añadió en la cola de usuarios pendientes."));
        
            // Aceptamos al usuario pendiente
            set_caller(alice);
            let result = contract.procesar_siguiente_usuario_pendiente(true);
            assert_eq!(result, Ok(String::from("Usuario agregado exitosamente.")));
        
            // Usuario registrado intenta ingresar a la elección
            set_caller(charlie);
            let result = contract.ingresar_a_eleccion_privado(eleccion_id, tipo_usuario.clone());
            assert_eq!(result, Ok(String::from("Ingresó a la elección correctamente Pendiente de aprobacion del Administrador")));
        
            
            // Limpiamos el estado de usuarios rechazados para continuar con el test
            contract.elecciones[0].usuarios_rechazados.clear();
            contract.elecciones[0].usuarios_pendientes.clear();
        

        }

        #[ink::test]
    fn test_iniciar_votacion_privado() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

        // Crear el contrato con el administrador
        let mut contrato = TrabajoFinal::new();

        // Añadir una elección de prueba
        contrato.elecciones.push(Eleccion {
            id: 1,
            candidatos: vec![],
            votantes: vec![],
            usuarios_pendientes: vec![],
            usuarios_rechazados: vec![],
            votacion_iniciada: false,
            fecha_inicio: 50,
            fecha_final: 150,
            resultados: None,
        });

        // Caso 1: No es administrador
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        assert_eq!(
            contrato.iniciar_votacion_privado(1),
            Err(String::from("No eres el administrador."))
        );

        // Caso 2: Elección no encontrada
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice); // Restaurar administrador
        assert_eq!(
            contrato.iniciar_votacion_privado(2),
            Err(String::from("No existe una elección con ese id."))
        );

        // Caso 3: Votación ya finalizó
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(200);
        assert_eq!(
            contrato.iniciar_votacion_privado(1),
            Err(String::from("La votación ya finalizó."))
        );

        // Caso 4: Votación ya inició
        ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(100); // Restaurar el timestamp del bloque
        contrato.elecciones[0].votacion_iniciada = true;
        assert_eq!(
            contrato.iniciar_votacion_privado(1),
            Err(String::from("La votación ya inició."))
        );

        // Caso 5: Todavía no es la fecha para la votación
        contrato.elecciones[0].votacion_iniciada = false;
        ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(30); // Cambiar el timestamp del bloque
        assert_eq!(
            contrato.iniciar_votacion_privado(1),
            Err(String::from("Todavía no es la fecha para la votación."))
        );

        // Caso 6: Se inició la votación exitosamente
        ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(100); // Restaurar el timestamp del bloque
        assert_eq!(
            contrato.iniciar_votacion_privado(1),
            Ok(String::from("Se inició la votación exitosamente."))
        );
        assert!(contrato.elecciones[0].votacion_iniciada);
    }
        
        #[ink::test]
        fn test_obtener_candidatos_eleccion_por_id(){
            let administrador: AccountId = AccountId::from([0x1; 32]);
            let generador_reportes: AccountId = AccountId::from([0x2; 32]);
            set_caller(administrador);
            
            let mut contrato = TrabajoFinal::new();
            assert!(contrato.asignar_generador_reportes(generador_reportes).is_ok());
            assert!(contrato.generador_reportes.is_some_and(|id| id == generador_reportes));
            
            set_caller(generador_reportes);
            let mut eleccion = setup_eleccion();
            let eleccion_id = eleccion.id;
            eleccion.fecha_final = contrato.env().block_timestamp();
            
            contrato.elecciones.push(eleccion);
            advance_block::<ink::env::DefaultEnvironment>();
            
            let resultado = contrato.obtener_candidatos_eleccion_por_id_privado(eleccion_id);
            
            // Verificar el resultado y manejar el error
            assert!(
                resultado.is_ok(),
                "Error al obtener candidatos para la elección: {:?}",
                resultado.unwrap_err().to_string()
            );
            }
    }

}    




