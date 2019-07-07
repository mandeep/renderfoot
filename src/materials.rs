use nalgebra::core::Vector3;
use rand::rngs;

use hitable::HitRecord;
use ray::{pick_sphere_point, Ray};
use texture::Texture;


/// The Material trait is responsible for giving a color to the object implementing the trait
pub trait Material: Send + Sync {
    fn box_clone(&self) -> Box<Material>;

    fn scatter(&self,
               ray: &Ray,
               record: &HitRecord,
               rng: &mut rand::rngs::ThreadRng) -> Option<(Vector3<f32>, Ray)>;

    fn emitted(&self, u: f32, v: f32, p: &Vector3<f32>) -> Vector3<f32> {
        Vector3::zeros()
    }
}


#[derive(Clone)]
pub struct Diffuse {
    pub albedo: Box<dyn Texture>
}



impl Diffuse {
    /// Create a new Diffuse material with the given albedo
    ///
    /// albedo is a Vector3 of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0.
    pub fn new<T: Texture + 'static>(albedo: T) -> Diffuse {
        let albedo = Box::new(albedo);
        Diffuse { albedo: albedo }
    }
}


impl Material for Diffuse {
    /// Create a new Diffuse Material on the heap
    fn box_clone(&self) -> Box<Material> {
        Box::new((*self).clone())
    }

    /// Retrieve the color of the given material
    ///
    /// For spheres, the center of the sphere is given by the record.point
    /// plus the record.normal. We add a random point from the unit sphere
    /// to uniformly distribute hit points on the sphere. The target minus
    /// the record.point is used to determine the ray that is being reflected
    /// from the surface of the material.
    fn scatter(&self,
               ray: &Ray,
               record: &HitRecord,
               rng: &mut rand::rngs::ThreadRng) -> Option<(Vector3<f32>, Ray)> {

        let target: Vector3<f32> = record.point + record.normal + pick_sphere_point(rng);
        Some((self.albedo.value(record.u, record.v, &record.point), Ray::new(record.point, target - record.point, ray.time)))
}
}

/// Compute the reflect vector given the light vector and the normal vector of the surface
///
/// The law of reflection tells us that the angle between the indicent ray
/// and the normal vector of the hit point is equal to the angle between
/// the reflected ray and the normal vector of the hit point.
///
/// For derivation see Section 10.4.2 in Mathematical and Computer Programming
/// Techniques for Computer Graphics by Peter Comininos.
fn reflect(v: &Vector3<f32>, n: &Vector3<f32>) -> Vector3<f32> {
    v - 2.0 * v.dot(&n) * n
}


/// Compute the refract vector given the light vector, normal vector, and refractive_index
///
/// In dielectric materials some light is reflected and some refracted. We can use
/// Snell's Law to compute the direction of the refracted light.
///
/// For derivation see Section 10.4.3 in Mathematical and Computer Programming
/// Techniques for Computer Graphics by Peter Comininos.
fn refract(v: &Vector3<f32>, n: &Vector3<f32>, refractive_index: f32) -> Option<Vector3<f32>> {
    let uv: Vector3<f32> = v.normalize();
    let direction: f32 = uv.dot(&n);
    let discriminant: f32 = 1.0 - refractive_index * refractive_index * (1.0 - direction * direction);

    if discriminant > 0.0 {
        return Some(refractive_index * (uv - n * direction) - n * discriminant.sqrt());
    }
    None
}

/// Determine the reflectivity amount based on the angle
///
/// In objects like glass, reflectivity varies with the view angle. Schlick's
/// approximation is used to compute the Fresnel factor in the specular reflection.
///
/// For derivation see Section 10.10.3 in Mathematical and Computer Programming
/// Techniques for Computer Graphics by Peter Comininos and
/// https://en.wikipedia.org/wiki/Schlick's_approximation.
fn schlick(cosine: f32, reference_index: f32) -> f32 {
    let r0: f32 = (1.0 - reference_index) / (1.0 + reference_index);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}


#[derive(Clone)]
pub struct Reflective {
    pub albedo: Vector3<f32>,
    pub fuzz: f32
}


impl Reflective {
    /// Create a new Reflective material for objects that reflect light only
    ///
    /// albedo is a Vector3 of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0. fuzz accounts
    /// for the fuzziness of the reflections due to the size of the sphere.
    /// Generally, the larger the sphere, the fuzzier the reflections will be.
    pub fn new(albedo: Vector3<f32>, fuzz: f32) -> Reflective {
        Reflective { albedo: albedo, fuzz: fuzz }
    }

}


impl Material for Reflective {
    /// Create a new Reflective material on the heap
    fn box_clone(&self) -> Box<Material> {
        Box::new((*self).clone())
    }

    /// Retrieve the color of the given material
    ///
    /// For spheres, the center of the sphere is given by the record.point
    /// plus the record.normal. We add a random point from the unit sphere
    /// to uniformly distribute hit points on the sphere. A fuzziness
    /// factor is also added in to account for the reflection fuzz due to
    /// the size of the sphere. The target minus the record.point is used
    /// to determine the ray that is being reflected from the surface of the material.
    fn scatter(&self,
               ray: &Ray,
               record: &HitRecord,
               rng: &mut rand::rngs::ThreadRng) -> Option<(Vector3<f32>, Ray)> {

        let reflected: Vector3<f32> = reflect(&ray.direction.normalize(), &record.normal);
        let scattered = Ray::new(record.point, reflected + self.fuzz * pick_sphere_point(rng), ray.time);
        if scattered.direction.dot(&record.normal) > 0.0 {
            return Some((self.albedo, scattered));
        }
        None
    }
}


#[derive(Clone)]
pub struct Refractive {
    pub albedo: Vector3<f32>,
    pub refractive_index: f32,
    pub fuzz: f32
}


impl Refractive {
    /// Create a new Refractive material for objects that both reflect and transmit light
    ///
    /// albedo is a Vector3 of the RGB values assigned to the material
    /// where each value is a float between 0.0 and 1.0. index determines
    /// how much of the light is refracted when entering the material.
    /// fuzz accounts for the fuzziness of the reflections due to the size of the sphere.
    /// Generally, the larger the sphere, the fuzzier the reflections will be.
    pub fn new(albedo: Vector3<f32>, index: f32, fuzz: f32) -> Refractive {
        Refractive { albedo: albedo, refractive_index: index, fuzz: fuzz }
    }
}


impl Material for Refractive {
    /// Create a new Refractive Material on the heap
    fn box_clone(&self) -> Box<Material> {
        Box::new((*self).clone())
    }

    /// Retrieve the color of the given material
    ///
    /// For spheres, the center of the sphere is given by the record.point
    /// plus the record.normal. We add a random point from the unit sphere
    /// to uniformly distribute hit points on the sphere. A fuzziness
    /// factor is also added in to account for the reflection fuzz due to
    /// the size of the sphere. The target minus the record.point is used
    /// to determine the ray that is being reflected from the surface of the material.
    ///
    /// See Peter Shirley's Ray Tracing in One Weekend for an overview of refractive
    /// scattering and Section 10.3.2 in Mathematical and Computer Programming
    /// Techniques for Computer Graphics by Peter Comininos.
    fn scatter(&self,
               ray: &Ray,
               record: &HitRecord,
               rng: &mut rand::rngs::ThreadRng) -> Option<(Vector3<f32>, Ray)> {

        let reflected: Vector3<f32> = reflect(&ray.direction.normalize(), &record.normal);
        let incident: f32 = ray.direction.dot(&record.normal);

        let (outward_normal, refractive_index, cosine) = if incident > 0.0 {
               (-record.normal,
                self.refractive_index,
                self.refractive_index * ray.direction.dot(&record.normal) / ray.direction.norm())
            } else {
               (record.normal,
                1.0 / self.refractive_index,
                -ray.direction.dot(&record.normal) / ray.direction.norm())

            };

        let refracted = refract(&ray.direction, &outward_normal, refractive_index);
        let reflect_probability = match refracted {
            Some(_) => schlick(cosine, self.refractive_index),
            None => 1.0
        };

        if rand::random::<f32>() < reflect_probability {
            return Some((self.albedo,
                         Ray::new(record.point, reflected + self.fuzz * pick_sphere_point(rng), ray.time)
                         ));
        } else {
            return Some((self.albedo, Ray::new(record.point,
                                               refracted.unwrap() +
                                                   self.fuzz * pick_sphere_point(rng), ray.time)
                         ));
        }
    }
}


#[derive(Clone)]
pub struct Light {
    pub emit: Box<dyn Texture>
}


impl Light {
    pub fn new<T: Texture + 'static>(emit: T) -> Light {
        let emit = Box::new(emit);
        Light { emit: emit }
    }
}


impl Material for Light {
    fn box_clone(&self) -> Box<Material> {
        Box::new((*self).clone())
    }

    fn scatter(&self,
               ray: &Ray,
               record: &HitRecord,
               rng: &mut rand::rngs::ThreadRng) -> Option<(Vector3<f32>, Ray)> {
        None
    }

    fn emitted(&self, u: f32, v: f32, p: &Vector3<f32>) -> Vector3<f32> {
        self.emit.value(u, v, &p)
    }
}
