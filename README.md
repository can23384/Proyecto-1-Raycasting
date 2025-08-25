# Proyecto Raycasting FPS (Rust + raylib)

Un mini–FPS estilo **Wolfenstein 3D** hecho con **Rust** y **raylib**.  
Incluye:
- Raycasting de paredes con texturas  
- Sprites 2D para enemigos y pickups  
- Armas con rarezas y munición  
- HUD completo  
- Minimap  
- Menú inicial y pantalla de victoria  

---

## 🚀 Requisitos

- **Rust** (versión estable) + cargo  
- **Windows / Linux / macOS**  
- **raylib-rs 5.5.1** (ya incluido).  

⚠️ **Importante:**  
No habilites features inexistentes como audio.  
Si ves errores tipo `RaylibAudio cannot be instantiated more than once`, asegúrate de crear **una sola instancia global** de `RaylibAudio`.

---

## ⚙️ Cómo compilar y ejecutar

```bash
# 1) Clonar / abrir el proyecto
# 2) Ejecutar en modo desarrollo
cargo run

# (opcional) en release
cargo run --release
```

## 🎮 Controles

W/S → Avanzar / retroceder

A/D → Girar

1..5 → Seleccionar slot

0 → Mano vacía (puñetazo)

Espacio → Disparar / puñetazo

R → Recargar

E → Interactuar (pickups, cofres)

F → Consumir consumible del slot (con cooldown)

M → Expandir / contraer minimapa

ESC → Salir

Mouse Click / ENTER / SPACE → Iniciar en menú
