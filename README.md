# Proyecto 2: Diorama con Raytracing

Diorama estilo Minecraft renderizado con **raytracing en CPU**, desarrollado en Rust.

La escena es una isla generada proceduralmente: tiene un lago con agua transparente, playas de arena, montañas de piedra con cimas de hielo, árboles y lámparas de glowstone. Todo el raytracing (rayos, intersecciones, luces, sombras, reflexión y refracción) está hecho a mano. Raylib solo se usa para abrir la ventana, leer el teclado/mouse y mostrar la imagen ya calculada.

## Video

[Ver video del diorama](PEGAR_LINK_DEL_VIDEO_AQUI)

## Ejecución

Es necesario tener Rust instalado. Desde la carpeta del proyecto:

```
cargo run --release
```

## Controles

- `A` / `D` o flechas izquierda/derecha: rotar el diorama
- Click izquierdo + arrastrar: rotar con el mouse
- `W` / `S` o rueda del mouse: acercar y alejar la cámara
- Flechas arriba/abajo: inclinar la cámara
- `Espacio`: activar/desactivar la rotación automática
- `N`: cambiar entre día y noche
- `R`: generar un terreno nuevo
- `1` / `2` / `3`: calidad baja / media / alta

## Características implementadas

### Rotación y zoom de la cámara
La cámara orbita alrededor del centro del diorama usando coordenadas esféricas (yaw, pitch y distancia). Se puede rotar con teclado o mouse, y acercar o alejar con `W`/`S` o la rueda. Por defecto el diorama gira solo.

### Materiales (9)
Cada material tiene su propia textura y sus propios valores de albedo, specular, reflectividad, transparencia e índice de refracción (`src/material.rs`).

| Material  | Textura | Detalle |
|-----------|---------|---------|
| Pasto     | Textura arriba y otra a los lados | Material difuso |
| Tierra    | Propia | Material difuso |
| Piedra    | Propia + mapa normal | Relieve con normal map |
| Arena     | Propia | Playas y fondo del lago |
| Agua      | Propia + mapa normal animado | Refracción, reflexión y transparencia |
| Madera    | Anillos arriba, corteza a los lados | Troncos y postes |
| Hojas     | Propia | Copas de los árboles |
| Hielo     | Propia | Muy reflectivo, cimas de las montañas |
| Glowstone | Propia | Emisivo, funciona como luz |

Las texturas son de 16x16 pixeles y se generan por código al iniciar, porque no se pueden usar librerías para cargar imágenes.

### Refracción
El agua del lago usa refracción con la ley de Snell (índice 1.33). Se nota en el fondo de arena, que se ve desviado a través del agua, y en los lados del diorama, donde se ve el corte del lago. Cuando el rayo sale del agua hacia el aire también se refracta, y si el ángulo es muy grande se refleja (reflexión interna total). Dentro del agua el color se va tiñendo de azul según la distancia.

### Reflexión
El hielo de las cimas (reflectividad 0.4) y el agua (0.25) reflejan el cielo, el sol y el terreno.

### Mapas normales
La piedra tiene un mapa normal sacado de la misma altura que genera su textura, así el relieve se nota con la luz del sol, sobre todo en los lados del diorama. El agua también tiene un mapa normal de olas que se mueve con el tiempo, lo que hace que los reflejos y la refracción se muevan.

### Material emisivo
El glowstone suma su propia luz a su color, y además cada bloque de glowstone funciona como una luz puntual que ilumina (con sombras) lo que tiene cerca. En modo noche (`N`) se nota muchísimo.

### Skybox
Skybox de 6 caras (cubemap). Cada cara es una textura de 128x128 que se pinta al iniciar con degradado, nubes y sol. Hay otro skybox para la noche con estrellas y luna. El sol de la escena sale de la misma dirección donde está pintado en el cielo.

### Generación procedural de terreno
El terreno es de **24x24 bloques** (más que el mínimo de 16x16) y se genera con ruido fractal (value noise con varias octavas), implementado a mano en `src/noise.rs`:

- La altura de cada columna sale del ruido.
- El nivel del agua se calcula a partir de la parte más baja, así siempre hay un lago.
- Según la altura se decide si arriba va arena, pasto, piedra o hielo.
- Los árboles y las lámparas se colocan en lugares de pasto elegidos con la semilla, sin quedar pegados.
- Con `R` se genera otro terreno con una semilla diferente.

## Optimización y programación paralela

- **Hilos con la librería estándar:** la imagen se divide en grupos de 4 filas y se usan tantos hilos como núcleos tenga la computadora (`std::thread::scope`). Los hilos van agarrando trabajos de una cola compartida con `Mutex`, así ningún hilo se queda sin hacer nada si otra parte de la imagen es más pesada.
- **Recorrido DDA en 3D:** en lugar de probar cada rayo contra todos los cubos, el rayo avanza bloque por bloque dentro de la cuadrícula del mundo (la misma idea del raycasting del proyecto 1, pero en 3D).
- **Recorte con la caja del mundo:** si un rayo no toca la caja que encierra el diorama, va directo al skybox sin recorrer nada.
- **Luces cercanas:** las lámparas solo se calculan si están a menos de 9 bloques del punto.
- **Límite de rebotes:** reflexión y refracción tienen un máximo de 3 rebotes.
- **Resolución interna:** la imagen se calcula más pequeña y se estira a la ventana. La calidad se cambia con `1`, `2` y `3`.
- Todo lo que no cambia (texturas, skybox, mundo) se calcula una sola vez al iniciar, no en cada frame.

## Estructura

```
src/
├── main.rs       ventana, controles y loop principal
├── render.rs     render en paralelo
├── raytracer.rs  trace, sombreado, reflexión y refracción
├── world.rs      mundo de bloques, generación de terreno y DDA
├── noise.rs      funciones de ruido
├── texture.rs    texturas y mapas normales
├── material.rs   materiales y sus parámetros
├── skybox.rs     cubemap del cielo (día y noche)
├── camera.rs     cámara orbital
├── scene.rs      junta todo lo de la escena
└── vec3.rs       vectores
```
