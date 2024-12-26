//! DTOs (Data Transfer Objects) are used to transfer data between the server and the client, all incoming and outgoing
//! data should be one of such DTOs.

use errors::ServerResult;

pub mod post_dto;
pub mod session_dto;
pub mod user_dto;

/// A trait that defines the conversion between a DTO and a model.
/// A Model that corresponds to a DTO should implement this trait.
#[allow(unused)]
pub trait Model: Sized {
    type DTO: DTO<Model = Self>;

    fn to_dto(self) -> ServerResult<Self::DTO> {
        return Self::DTO::from_model(self);
    }

    fn from_dto(dto: Self::DTO) -> ServerResult<Self> {
        return dto.to_model();
    }
}

/// A trait that defines the conversion between a DTO and a model.
/// A DTO that corresponds to a model should implement this trait.
#[allow(unused, clippy::upper_case_acronyms)]
pub trait DTO: Sized {
    type Model: Model<DTO = Self>;

    fn to_model(self) -> ServerResult<Self::Model>;

    fn from_model(model: Self::Model) -> ServerResult<Self>;
}

#[allow(unused)]
pub trait ModelVectorExt: Sized {
    type DTO;

    fn to_dtos(self) -> ServerResult<Vec<Self::DTO>>;

    fn from_dtos(dtos: Vec<Self::DTO>) -> ServerResult<Self>;
}

#[allow(unused)]
pub trait DTOVectorExt: Sized {
    type Model;

    fn to_models(self) -> ServerResult<Vec<Self::Model>>;

    fn from_models(models: Vec<Self::Model>) -> ServerResult<Self>;
}

#[allow(unused)]
impl<T: Model> ModelVectorExt for Vec<T> {
    type DTO = T::DTO;

    fn to_dtos(self) -> ServerResult<Vec<Self::DTO>> {
        self.into_iter().map(T::to_dto).collect()
    }

    fn from_dtos(dtos: Vec<Self::DTO>) -> ServerResult<Self> {
        dtos.into_iter().map(T::from_dto).collect()
    }
}

#[allow(unused)]
impl<T: DTO> DTOVectorExt for Vec<T> {
    type Model = T::Model;

    fn to_models(self) -> ServerResult<Vec<Self::Model>> {
        self.into_iter().map(T::to_model).collect()
    }

    fn from_models(models: Vec<Self::Model>) -> ServerResult<Self> {
        models.into_iter().map(T::from_model).collect()
    }
}
