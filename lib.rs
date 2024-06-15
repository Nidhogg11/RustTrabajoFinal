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
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
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
        usuario:Usuario,
        voto_emitido:bool,
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    struct Candidato
    {
        usuario:Usuario,
        candidatura:String,
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    struct CandidatoConteo
    {
        candidato:Candidato,
        votos_totales:u32,
    }

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    struct Eleccion
    {
        id:u32,
        candidatos:Vec<CandidatoConteo>,
        votantes:Vec<Votante>,
        usuarios:Vec<AccountId>,
        votacion_iniciada:bool,
        fecha_inicio:i64,
        fecha_final:i64,
    }

    impl Eleccion
    {
        fn contiene_usuario(&self, id:AccountId) -> bool
        {
            self.usuarios.contains(&id)
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

        fn es_usuario_registrado(&self) -> bool
        {
            let id = self.env().caller();
            for usuario in self.usuarios.iter() {
                if usuario.id == id
                {
                    return true;
                }
            }
            return false;
        }

        fn es_usuario_pendiente(&self) -> bool
        {
            let id = self.env().caller();
            for usuario in self.usuarios_pendientes.iter() {
                if usuario.id == id
                {
                    return true;
                }
            }
            return false;
        }

        fn existe_eleccion(&self, eleccion_id:u32) -> bool
        {
            for eleccion in self.elecciones.iter() {
                if eleccion.id == eleccion_id
                {
                    return true;
                }
            }
            return false;
        }

        fn obtener_eleccion_por_id(&mut self, eleccion_id:u32) -> Option<&mut Eleccion>
        {
            for eleccion in self.elecciones.iter_mut() {
                if eleccion.id == eleccion_id
                {
                    return Some(eleccion);
                }
            }
            return None;
        }

        fn es_administrador(&self) -> bool
        {
            self.env().caller() == self.administrador
        }

        /// Usado por un administrador.
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
                usuarios: Vec::new(),
                votacion_iniciada:false,
                fecha_inicio: fecha_inicial_milisegundos.unwrap().and_utc().timestamp_millis(),
                fecha_final: fecha_final_milisegundos.unwrap().and_utc().timestamp_millis(),
            };
            self.elecciones.push(eleccion);

            return Ok(String::from("Eleccion creada exitosamente. Id de la elección: ") + &eleccion_id.to_string());
        }

        #[ink(message)]
        pub fn ingresar_a_eleccion(&mut self, eleccion_id:u32, tipo:TIPO_DE_USUARIO, candidatura:Option<String>) -> Result<String, String>
        {
            if !self.es_usuario_registrado() { return Err(ERRORES::USUARIO_NO_REGISTRADO.to_string()); }
            let id = self.env().caller();
            let option_eleccion = self.obtener_eleccion_por_id(eleccion_id);
            if option_eleccion.is_none() { return Err(String::from("No existe una elección con ese id.")); }
            
            let eleccion = option_eleccion.unwrap();
            if eleccion.contiene_usuario(id) { return Err(String::from("Ya está registrado en la elección.")); }

            match tipo
            {
                TIPO_DE_USUARIO::VOTANTE => {
                    return Ok(String::from("Ingresó a la elección correctamente como votante."));
                },
                TIPO_DE_USUARIO::CANDIDATO => 
                {
                    if candidatura.is_none() { return Err(String::from("Para ser candidato tiene que presentar una candidatura.")); }
                    
                    return Ok(String::from("Ingresó a la elección correctamente como candidato."));
                },
            }
        }

        /// Usado por un Administrador.
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

        /// Usado por un Administrador.
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
    }

    /*#[cfg(test)]
    mod tests {
        use super::*;
    }*/
}