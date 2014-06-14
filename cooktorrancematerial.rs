use vec3::Vec3;
use material::Material;
use std::f64::consts::PI;

pub struct CookTorranceMaterial {
    pub k_a: f64,            // Ambient coefficient
    pub k_d: f64,            // Diffuse coefficient
    pub k_s: f64,            // Local specular coefficient
    pub k_sg: f64,           // Global specular coefficient (mirror reflection)
    pub k_tg: f64,           // Global transmissive coefficient (refraction)
    pub ambient: Vec3,       // Ambient color
    pub diffuse: Vec3,       // Diffuse color
    pub transmission: Vec3,  // Transmissive color
    pub specular: Vec3,      // Specular color
    pub roughness: f64,      //
    pub gauss_constant: f64, //
    pub ior: f64             // Index of refraction
}

impl Material for CookTorranceMaterial {
    fn sample(&self, n: Vec3, i: Vec3, l: Vec3) -> Vec3 {
        let h = (l + i).unit();

        // Blinn-Phong approximation
        let ambient  = self.ambient.scale(self.k_a);
        let diffuse  = self.diffuse.scale(self.k_d).scale(n.dot(&l));

        // Specular calculations
        let n_dot_h = n.dot(&h);
        let n_dot_l = n.dot(&l);
        let v_dot_h = i.dot(&h);
        let n_dot_v = n.dot(&i);

        // Approximate Fresnel term, Schlick's approximation
        let n1 = 1.0;
        let n2 = self.ior;
        let f0 = ((n1 - n2) / (n1 + n2)).powf(2.0);
        let f = (1.0 - v_dot_h).powf(5.0) * (1.0 - f0) + f0;

        // Microfacet distribution
        let alpha = n_dot_h.acos();
        let d = self.gauss_constant * (-alpha / self.roughness.sqrt()).exp();

        // Geometric attenuation factor
        let g1 = (2.0 * n_dot_h * n_dot_v) / v_dot_h;
        let g2 = (2.0 * n_dot_h * n_dot_l) / v_dot_h;
        let g = g1.min(g2);

        let brdf = f * d * g / (n_dot_v * n_dot_l);

        self.specular.scale(self.k_s * brdf) + diffuse + ambient
    }

    fn is_reflective(&self) -> bool {
        self.k_sg > 0.0
    }

    fn is_refractive(&self) -> bool {
        self.k_tg > 0.0
    }

    fn global_specular(&self, color: &Vec3) -> Vec3 {
        color.scale(self.k_sg)
    }

    fn global_transmissive(&self, color: &Vec3) -> Vec3 {
        color.scale(self.k_tg)
    }

    fn transmission(&self) -> Vec3 {
        self.transmission
    }

    fn ior(&self) -> f64 {
        self.ior
    }
}