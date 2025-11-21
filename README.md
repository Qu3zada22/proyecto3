# 🌌 Sistema Solar Renderizado por Software

<img width="1019" height="705" alt="imagen" src="https://github.com/user-attachments/assets/a953450c-89ab-46f2-b76d-b3eec7a0174b" />


Este proyecto es una simulación interactiva de un sistema solar **renderizada 100% desde cero**, sin motores gráficos externos (solo `raylib` para ventana y entrada). Todos los triángulos, transformaciones, iluminación, órbitas y efectos visuales están implementados manualmente en Rust.

✅ Planetas texturizados proceduralmente  
✅ Sol con llamaradas, capas y glow  
✅ Fondo negro profundo con estrellas de colores  
✅ Nave espacial que sigue a la cámara  
✅ Sistema de cámaras 3D con warping instantáneo y evasión de colisiones  

---

## 🌠 Características

- 🌞 **Sol dinámico**: con núcleo, fotosfera, corona y llamaradas animadas.
- 🪐 **5 planetas únicos**: Mercurio (rocoso), Tierra (océanos, nubes, atmósfera), Marte (polvo y cráteres), Urano (bandas y brillo helado).
- 🛸 **Nave espacial 3D** modelada y renderizada, que sigue la cámara en tiempo real.
- 🌠 **Cielo estrellado**: 300 estrellas con colores variados (blancas, azules, amarillas).
- 🌀 **Cámara 3D avanzada**:
  - Movimiento libre (WASD + flechas + Q/E).
  - *Warping* instantáneo con animación suave (teclas `1`–`5`).
  - Detección de colisiones con cuerpos celestes.
- 📏 **Órbitas visibles** en el plano eclíptico.
- ⚡ **Alto rendimiento**: optimizado para mantener FPS estables incluso con todos los efectos activos.

---

## Video de Demostración

https://youtu.be/Ane6mZlRVgc


**Contenido del video**:
- Recorrido completo del sistema solar.
- Transiciones entre puntos de vista con *warping* animado.
- Primer plano de cada planeta mostrando sus texturas y rotación.
- Nave espacial siguiendo la cámara.
- Movimiento libre en 3D.

---

## Cómo ejecutarlo

### Pasos

1. **Clona el repositorio**
   ```bash
   git clone https://github.com/Qu3zada22/proyecto3.git
  
