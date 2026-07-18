# ============================================================

# 📚 RemapH V3

# 00_Indice.md

# ============================================================

## 🎯 Propósito

Este documento es el punto de entrada de la documentación oficial de RemapH V3.
No describe el funcionamiento interno de una capa específica.
Su objetivo es mostrar la estructura general del proyecto y servir como mapa para localizar rápidamente cualquier parte de la documentación.
Toda decisión de arquitectura deberá quedar documentada en alguno de los documentos indicados aquí.

---

# 📑 Índice

1. Objetivos del proyecto
2. Arquitectura general
3. Organización de la documentación
4. Organización del código
5. Estado del proyecto
6. Convenciones generales
7. Última actualización

---

# 🎯 1. Objetivos del proyecto

RemapH V3 es un remapeador avanzado de dispositivos de entrada para Windows, desarrollado con una arquitectura modular desde cero.
Su objetivo no es únicamente reasignar teclas, sino convertirse en una plataforma flexible para traducir cualquier entrada física en cualquier acción configurable por el usuario.

RemapH V3 busca convertirse en un remapeador profesional para Windows.
Los pilares del proyecto son:

• Simplicidad.
• Escalabilidad.
• Alto rendimiento.
• Arquitectura desacoplada.
• Mantenimiento sencillo.

Toda decisión futura deberá respetar estos principios.

---

# 🏛️ 2. Arquitectura general

El proyecto está dividido en cuatro capas completamente independientes.
Cada una posee una única responsabilidad.

```
                Usuario
                   │
        ─────────────────────

                  UI
     Interfaz e interacción

                   │

                 Core
  Comunicación y reglas comunes

                   │

               Runtime
      Motor del remapeador

                   │

               Platform
 Integración con Windows y hardware
```

Cada capa únicamente conoce aquello que necesita para cumplir su responsabilidad.
Las capas superiores nunca ejecutan responsabilidades de las inferiores.
Las capas inferiores nunca conocen detalles internos de las superiores.

---

## 🟦 UI

Responsable de toda la experiencia del usuario.
Incluye:

• Interfaz.
• Componentes.
• Ventanas.
• Edición de perfiles.
• Barras.
• Controles.

No contiene lógica de remapeo.

---

## 🟨 Core

Responsable de la comunicación interna.
Incluye:

• Eventos.
• Contexto compartido.
• Reglas comunes.
• Infraestructura de comunicación.

No conoce la interfaz ni el sistema operativo.

---

## 🟩 Runtime

Responsable del funcionamiento del remapeador.
Incluye:

• Compilador.
• Caché.
• Motor de ejecución.
• Resolución de acciones.
• Gestión de remapeos.

No conoce controles visuales.
No conoce Windows.
Trabaja únicamente con modelos internos.

---

## 🟥 Platform

Responsable de comunicar RemapH con el sistema operativo.
Incluye:

• Tauri.
• Interception.
• APIs de Windows.
• Hardware.
• Entrada física.
• Salida física.

No contiene reglas del remapeador.
Únicamente traduce información entre el sistema operativo y el Runtime.

---

# 📂 3. Organización de la documentación

```
Documentacion/

00_Indice.md

01_UI.md
02_Core.md
03_Runtime.md
04_Platform.md

05_Estilos.md
06_Principios.md
07_Pendientes.md
```

Cada documento responde una única pregunta.

| Documento     | Contenido                           |
| ------------- | ----------------------------------- |
| 00_Indice     | Visión general del proyecto.        |
| 01_UI         | Arquitectura de toda la interfaz.   |
| 02_Core       | Comunicación interna y eventos.     |
| 03_Runtime    | Motor del remapeador.               |
| 04_Platform   | Integración con Windows.            |
| 05_Estilos    | Identidad visual del proyecto.      |
| 06_Principios | Reglas arquitectónicas permanentes. |
| 07_Pendientes | Funcionalidades futuras.            |

---

# 📁 4. Organización del código

```
Frontend (TypeScript)
↓
UI
↓
Core

══════════════════════════════

Backend (Rust)
↓
Runtime
↓
Platform
↓
Windows
```

La separación por lenguaje es una decisión técnica.
La separación por capas es una decisión arquitectónica.
La arquitectura siempre tiene prioridad.

---

# 🟢 5. Estado del proyecto

| Capa          | Estado             |
| ------------- | ------------------ |
| UI            | 🟡 En construcción |
| Core          | 🟡 En construcción |
| Runtime       | 🟢 Base estable    |
| Platform      | 🟢 Base estable    |
| Estilos       | 🟡 En crecimiento  |
| Documentación | 🟡 En construcción |

---

# 📐 6. Convenciones generales

Todo archivo debe tener una única responsabilidad.
Toda carpeta debe agrupar responsabilidades relacionadas.
Toda comunicación entre componentes debe respetar la arquitectura del proyecto.
Antes de crear un nuevo módulo debe responderse una pregunta:

> ¿A qué capa pertenece?

Si la respuesta no es evidente, probablemente la responsabilidad aún no está bien definida.

---

# 📝 7. Última actualización

Versión de la arquitectura:
Arquitectura V2
Cambios principales:

• Se reemplazó la división Frontend / Backend por una arquitectura basada en responsabilidades.
• Se definieron las cuatro capas oficiales del proyecto.
• Se separó el Runtime de Platform.
• Se inició la documentación modular del proyecto.