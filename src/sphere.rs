use std::f32::consts::PI;
use std::sync::Arc;

use glam::Vec3;

use aabb::AABB;
use hitable::{HitRecord, Hitable};
use materials::Material;
use ray::Ray;

#[derive(Clone)]
pub struct Sphere {
    pub start_center: Vec3,
    pub end_center: Vec3,
    pub radius: f32,
    pub material: Arc<dyn Material>,
    pub start_time: f32,
    pub end_time: f32,
}

impl Sphere {
    /// Create a new sphere to place into the world
    ///
    /// We use the 'static lifetime so that we can create a Arc material
    /// within the function rather than having to pass a Arc material
    /// as an input parameter.
    pub fn new<M: Material + 'static>(start_center: Vec3,
                                      end_center: Vec3,
                                      radius: f32,
                                      material: M,
                                      start_time: f32,
                                      end_time: f32)
                                      -> Sphere {
        let material = Arc::new(material);
        Sphere { start_center,
                 end_center,
                 radius,
                 material,
                 start_time,
                 end_time }
    }

    pub fn center(&self, time: f32) -> Vec3 {
        self.start_center
        + ((time - self.start_time) / (self.end_time - self.start_time))
          * (self.end_center - self.start_center)
    }
}

fn get_sphere_uv(p: &Vec3) -> (f32, f32) {
    let phi = p.z().atan2(p.x());
    let theta = p.y().asin();
    let u = 1.0 - (phi + PI) / (2.0 * PI);
    let v = (theta + PI / 2.0) / PI;
    (u, v)
}

impl Hitable for Sphere {
    /// Determine if the given ray intersects with a point on the sphere
    ///
    /// The equation is quadratic in terms of t. We solve for t looking for
    /// a real root. No real roots signifies a miss, one real root signifies
    /// a hit at the boundary of the sphere, and two real roots signify a
    /// ray hitting one point on the sphere and leaving through another point.
    fn hit(&self, ray: &Ray, position_min: f32, position_max: f32) -> Option<HitRecord> {
        let sphere_center: Vec3 = ray.origin - self.center(ray.time);
        let a: f32 = ray.direction.dot(ray.direction);
        let b: f32 = sphere_center.dot(ray.direction);
        let c: f32 = sphere_center.dot(sphere_center) - (self.radius * self.radius);
        let discriminant: f32 = b * b - a * c;

        // checking the discriminant is a fast way to determine if the root is real
        if discriminant > 0.0 {
            let first_root: f32 = (-b - discriminant.sqrt()) / a;
            let second_root: f32 = (-b + discriminant.sqrt()) / a;
            let roots = vec![first_root, second_root];

            for root in roots {
                if root > position_min && root < position_max {
                    let point = ray.point_at_parameter(root);
                    let normal = (point - self.center(ray.time)) / self.radius;
                    let (u, v) = get_sphere_uv(&normal);
                    return Some(HitRecord::new(root,
                                               u,
                                               v,
                                               point,
                                               normal,
                                               normal,
                                               self.material.clone()));
                }
            }
        }
        None
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        let radius = Vec3::new(self.radius, self.radius, self.radius);
        let min0 = self.center(t0) - radius;
        let max0 = self.center(t0) + radius;
        let min1 = self.center(t1) - radius;
        let max1 = self.center(t1) + radius;

        let small = AABB::from(min0, max0);
        let big = AABB::from(min1, max1);

        Some(small.surrounding_box(&big))
    }
}
