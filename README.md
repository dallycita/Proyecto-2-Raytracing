# Proyecto 2: Diorama con Raytracing

Diorama estilo Minecraft renderizado con **raytracing en CPU**, desarrollado en Rust.

La escena es una isla generada proceduralmente de 32x32 bloques con un lago, playas, montañas con cimas de hielo, una casita con ventanas de vidrio y chimenea, un muelle que entra al lago, un charco de lava entre las rocas, árboles, arbustos y lámparas de glowstone. Se puede ver de día, al atardecer y de noche.

Todo el raytracing (rayos, intersecciones, luces, sombras, reflexión y refracción) está hecho a mano. Raylib solo se usa para abrir la ventana, leer el teclado/mouse y mostrar la imagen ya calculada.

## Video

[Ver video del diorama](PEGAR_LINK_DEL_VIDEO_AQUI)

## Ejecución

Es necesario tener Rust instalado. Desde la carpeta del proyecto:

```
cargo run --release
```

## Controles

- `A` / `D`, flechas izquierda/derecha o click + arrastrar: rotar el diorama
- `W` / `S` o rueda del mouse: acercar y alejar la cámara
- Flechas arriba/abajo: inclinar la cámara
- `Espacio`: activar/desactivar la rotación automática
- `N`: cambiar entre día, atardecer y noche
- `R`: generar un terreno nuevo
- `1` / `2` / `3` / `4`: calidad (de más rápido a mejor calidad)
- `V`: grabar el video automáticamente

Cuando la cámara se queda quieta, la imagen se va refinando sola (el contador de "muestras" sube hasta 32).

## Características implementadas

### Rotación y zoom de la cámara
La cámara orbita alrededor del centro del diorama usando coordenadas esféricas (yaw, pitch y distancia). Se puede rotar con teclado o mouse, y acercar o alejar con `W`/`S` o la rueda.

### Materiales (13)
Cada material tiene su propia textura y sus propios valores de albedo, specular, reflectividad, transparencia e índice de refracción (`src/material.rs`).

| Material    | Dónde aparece | Detalle |
|-------------|---------------|---------|
| Pasto       | Terreno | Textura arriba y otra a los lados |
| Tierra      | Debajo del pasto | Difuso |
| Piedra      | Montañas y base del diorama | Mapa normal |
| Arena       | Playas y fondo del lago | Difuso |
| Agua        | Lago | Refracción, reflexión con Fresnel, mapa normal animado |
| Madera      | Troncos, esquinas de la casa, postes | Anillos arriba, corteza a los lados |
| Hojas       | Árboles y arbustos | Difuso |
| Hielo       | Cimas de las montañas | Reflexión y algo de transparencia |
| Glowstone   | Lámparas | Emisivo, ilumina alrededor |
| Tablas      | Paredes y techo de la casa, muelle | Mapa normal |
| Vidrio      | Ventanas de la casa | Refracción (índice 1.5) |
| Lava        | Charco en las rocas | Emisivo animado, ilumina alrededor |
| Cobblestone | Piso y chimenea de la casa | Mapa normal |

Las texturas son de 16x16 pixeles y se generan por código al iniciar, porque no se pueden usar librerías para cargar imágenes.

### Refracción
- **Agua (índice 1.33):** a través del lago se ve el fondo de arena y los postes del muelle desviados. Cuando el rayo sale del agua también se refracta, y si el ángulo es muy grande se refleja (reflexión interna total). Dentro del agua el color se va tiñendo de azul según la distancia.
- **Vidrio (índice 1.5):** las ventanas de la casa dejan ver el interior. De noche la luz de la lámpara de adentro sale por las ventanas.

### Reflexión
El hielo de las cimas y el agua reflejan el cielo, el sol y el terreno. En el agua y el vidrio se usa la aproximación de Fresnel: vistos de lado reflejan más.

### Mapas normales
La piedra, las tablas y el cobblestone tienen mapas normales sacados de la misma altura que genera su textura, así el relieve se nota con la luz. El agua tiene un mapa normal de olas que se mueve con el tiempo.

### Materiales emisivos
El glowstone y la lava suman su propia luz a su color, y además cada bloque emisivo funciona como una luz puntual que ilumina (con sombras suaves) lo que tiene cerca. Hay lámparas en el pasto, al final del muelle y dentro de la casa. De noche se nota muchísimo.

### Skybox
Skybox de 6 caras (cubemap) de 128x128 por cara. Hay tres: día (nubes y sol), atardecer (horizonte naranja y sol bajo) y noche (estrellas y luna). La luz de la escena sale de la misma dirección donde está el sol o la luna.

### Generación procedural de terreno
El terreno es de **32x32 bloques** y se genera con ruido fractal (value noise con varias octavas), implementado a mano en `src/noise.rs`:

- La altura de cada columna sale del ruido.
- El nivel del agua se calcula a partir de la parte más baja, así siempre hay un lago.
- Según la altura se decide si arriba va arena, pasto, piedra o hielo.
- La casa se construye en el lugar más plano y el terreno se aplana debajo.
- El muelle se pone donde hay playa con más agua enfrente.
- La lava busca un hoyito entre las rocas.
- Árboles, lámparas y arbustos se reparten sin quedar pegados.
- Con `R` se genera otro terreno con una semilla diferente.

### Otros efectos
- **Oclusión ambiental** estilo Minecraft: las esquinas y rincones se ven más oscuros.
- **Sombras suaves** del sol y de las lámparas.
- **Antialiasing** progresivo cuando la cámara está quieta.
- **Tone mapping ACES y gamma:** los cálculos de luz se hacen en espacio lineal y al final se convierten para la pantalla.

## Optimización y programación paralela

- **Hilos con la librería estándar:** la imagen se divide en grupos de 4 filas y se usan tantos hilos como núcleos tenga la computadora (`std::thread::scope`). Los hilos van agarrando trabajos de una cola compartida con `Mutex`, así ninguno se queda sin hacer nada si otra parte de la imagen pesa más.
- **Recorrido DDA en 3D:** en lugar de probar cada rayo contra todos los cubos, el rayo avanza bloque por bloque dentro de la cuadrícula del mundo (la misma idea del raycasting del proyecto 1, pero en 3D).
- **Recorte con la caja del mundo:** si un rayo no toca la caja que encierra el diorama, va directo al skybox.
- **Luces cercanas:** las luces a más de 10 bloques ni se calculan.
- **Límite de rebotes:** máximo 4 rebotes de reflexión/refracción.
- **Acumulación de muestras:** en vez de lanzar muchos rayos por pixel en cada frame, se lanza uno y se va promediando con los anteriores mientras la cámara no se mueve.
- **Resolución interna:** la imagen se calcula más pequeña y se estira a la ventana. Se cambia con `1`-`4`.
- Las texturas, los skyboxes y el mundo se calculan una sola vez, no en cada frame.

## Cómo se grabó el video

Al presionar `V` el programa hace un recorrido automático: la cámara da una vuelta completa, se acerca y se aleja, y pasa por día, atardecer y noche. Cada frame se calcula con 6 muestras a 960x540 y se guarda como imagen PPM en la carpeta `frames/` (el formato PPM se escribe a mano, sin librerías).

Después las imágenes se juntan en un video con ffmpeg:

```
ffmpeg -framerate 30 -i frames/frame_%04d.ppm -c:v libx264 -pix_fmt yuv420p diorama.mp4
```

## Estructura

```
src/
├── main.rs       ventana, controles, loop principal y modo video
├── render.rs     render en paralelo y acumulación de muestras
├── raytracer.rs  trace, sombreado, reflexión y refracción
├── world.rs      mundo de bloques, generación del terreno, DDA y oclusión ambiental
├── noise.rs      funciones de ruido y números random
├── texture.rs    texturas y mapas normales
├── material.rs   materiales y sus parámetros
├── skybox.rs     cubemaps del cielo (día, atardecer y noche)
├── camera.rs     cámara orbital
├── scene.rs      junta todo lo de la escena y la iluminación
└── vec3.rs       vectores
```
