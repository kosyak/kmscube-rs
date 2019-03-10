use std::f32::consts::PI;

#[derive(Debug, Copy, Clone)]
pub struct ESMatrix([[f32; 4]; 4]);

impl Default for ESMatrix {
    fn default() -> Self {
        ESMatrix([[0_f32; 4]; 4])
    }
}

impl ESMatrix {
    pub fn m(self) -> [[f32; 4]; 4] {
        self.0
    }

    pub fn l(self) -> [f32; 16] {
        [
            self.0[0][0], self.0[0][1], self.0[0][2], self.0[0][3],
            self.0[1][0], self.0[1][1], self.0[1][2], self.0[1][3],
            self.0[2][0], self.0[2][1], self.0[2][2], self.0[2][3],
            self.0[3][0], self.0[3][1], self.0[3][2], self.0[3][3],
        ]
    }

    pub fn identity() -> ESMatrix {
        let mut result = ESMatrix::default();
        result.0[0][0] = 1.0;
        result.0[1][1] = 1.0;
        result.0[2][2] = 1.0;
        result.0[3][3] = 1.0;

        result
    }

    pub fn translate(self, tx: f32, ty: f32, tz: f32) -> ESMatrix {
        let mut result = self;
        result.0[3][0] += result.0[0][0] * tx + result.0[1][0] * ty + result.0[2][0] * tz;
        result.0[3][1] += result.0[0][1] * tx + result.0[1][1] * ty + result.0[2][1] * tz;
        result.0[3][2] += result.0[0][2] * tx + result.0[1][2] * ty + result.0[2][2] * tz;
        result.0[3][3] += result.0[0][3] * tx + result.0[1][3] * ty + result.0[2][3] * tz;

        result
    }

    pub fn rotate(self, angle: f32, x: f32, y: f32, z: f32) -> ESMatrix {
        let mag = (x * x + y * y + z * z).sqrt();

        if mag == 0.0 {
            return self;
        }

        let sin_angle = (angle * PI / 180.0).sin();
        let cos_angle = (angle * PI / 180.0).cos();

        let x = x / mag;
        let y = y / mag;
        let z = z / mag;

        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let yz = y * z;
        let zx = z * x;
        let xs = x * sin_angle;
        let ys = y * sin_angle;
        let zs = z * sin_angle;
        let one_minus_cos = 1.0 - cos_angle;

        let mut rot_mat = ESMatrix::default();
        rot_mat.0[0][0] = (one_minus_cos * xx) + cos_angle;
        rot_mat.0[0][1] = (one_minus_cos * xy) - zs;
        rot_mat.0[0][2] = (one_minus_cos * zx) + ys;
        rot_mat.0[0][3] = 0.0;

        rot_mat.0[1][0] = (one_minus_cos * xy) + zs;
        rot_mat.0[1][1] = (one_minus_cos * yy) + cos_angle;
        rot_mat.0[1][2] = (one_minus_cos * yz) - xs;
        rot_mat.0[1][3] = 0.0;

        rot_mat.0[2][0] = (one_minus_cos * zx) - ys;
        rot_mat.0[2][1] = (one_minus_cos * yz) + xs;
        rot_mat.0[2][2] = (one_minus_cos * zz) + cos_angle;
        rot_mat.0[2][3] = 0.0;

        rot_mat.0[3][0] = 0.0;
        rot_mat.0[3][1] = 0.0;
        rot_mat.0[3][2] = 0.0;
        rot_mat.0[3][3] = 1.0;

        ESMatrix::multiply(rot_mat, self)
    }

    pub fn multiply(src_a: ESMatrix, src_b: ESMatrix) -> ESMatrix {
        let mut result = ESMatrix::default();

        (0_usize..4_usize).for_each(|i| {
            result.0[i][0] =
                (src_a.0[i][0] * src_b.0[0][0]) +
                (src_a.0[i][1] * src_b.0[1][0]) +
                (src_a.0[i][2] * src_b.0[2][0]) +
                (src_a.0[i][3] * src_b.0[3][0]);

            result.0[i][1] =
                (src_a.0[i][0] * src_b.0[0][1]) +
                (src_a.0[i][1] * src_b.0[1][1]) +
                (src_a.0[i][2] * src_b.0[2][1]) +
                (src_a.0[i][3] * src_b.0[3][1]);

            result.0[i][2] =
                (src_a.0[i][0] * src_b.0[0][2]) +
                (src_a.0[i][1] * src_b.0[1][2]) +
                (src_a.0[i][2] * src_b.0[2][2]) +
                (src_a.0[i][3] * src_b.0[3][2]);

            result.0[i][3] =
                (src_a.0[i][0] * src_b.0[0][3]) +
                (src_a.0[i][1] * src_b.0[1][3]) +
                (src_a.0[i][2] * src_b.0[2][3]) +
                (src_a.0[i][3] * src_b.0[3][3]);
        });

        result
    }

    pub fn frustum(self, left: f32, right: f32, bottom: f32, top: f32, near_z: f32, far_z: f32) -> ESMatrix {
        let delta_x = right - left;
        let delta_y = top - bottom;
        let delta_z = far_z - near_z;
        let mut frust = ESMatrix::default();

        if near_z <= 0.0 || far_z <= 0.0 || delta_x <= 0.0 || delta_y <= 0.0 || delta_z <= 0.0 {
            return self;
        }

        frust.0[0][0] = 2.0 * near_z / delta_x;
        frust.0[0][1] = 0.0;
        frust.0[0][2] = 0.0;
        frust.0[0][3] = 0.0;

        frust.0[1][1] = 2.0 * near_z / delta_y;
        frust.0[1][0] = 0.0;
        frust.0[1][2] = 0.0;
        frust.0[1][3] = 0.0;

        frust.0[2][0] = (right + left) / delta_x;
        frust.0[2][1] = (top + bottom) / delta_y;
        frust.0[2][2] = -(near_z + far_z) / delta_z;
        frust.0[2][3] = -1.0;

        frust.0[3][2] = -2.0 * near_z * far_z / delta_z;
        frust.0[3][0] = 0.0;
        frust.0[3][1] = 0.0;
        frust.0[3][3] = 0.0;

        ESMatrix::multiply(frust, self)
    }

// pub fn es_scale(ESMatrix *result, GLfloat sx, GLfloat sy, GLfloat sz) {
//     result->m[0][0] *= sx;
//     result->m[0][1] *= sx;
//     result->m[0][2] *= sx;
//     result->m[0][3] *= sx;

//     result->m[1][0] *= sy;
//     result->m[1][1] *= sy;
//     result->m[1][2] *= sy;
//     result->m[1][3] *= sy;

//     result->m[2][0] *= sz;
//     result->m[2][1] *= sz;
//     result->m[2][2] *= sz;
//     result->m[2][3] *= sz;
// }


// esPerspective(ESMatrix *result, float fovy, float aspect, float nearZ, float farZ)
// {
//    GLfloat frustumW, frustumH;

//    frustumH = tanf( fovy / 360.0f * PI ) * nearZ;
//    frustumW = frustumH * aspect;

//    esFrustum( result, -frustumW, frustumW, -frustumH, frustumH, nearZ, farZ );
// }

// esOrtho(ESMatrix *result, float left, float right, float bottom, float top, float nearZ, float farZ)
// {
//     float       deltaX = right - left;
//     float       deltaY = top - bottom;
//     float       deltaZ = farZ - nearZ;
//     ESMatrix    ortho;

//     if ( (deltaX == 0.0f) || (deltaY == 0.0f) || (deltaZ == 0.0f) )
//         return;

//     esMatrixLoadIdentity(&ortho);
//     ortho.m[0][0] = 2.0f / deltaX;
//     ortho.m[3][0] = -(right + left) / deltaX;
//     ortho.m[1][1] = 2.0f / deltaY;
//     ortho.m[3][1] = -(top + bottom) / deltaY;
//     ortho.m[2][2] = -2.0f / deltaZ;
//     ortho.m[3][2] = -(nearZ + farZ) / deltaZ;

//     esMatrixMultiply(result, &ortho, result);
// }
}
