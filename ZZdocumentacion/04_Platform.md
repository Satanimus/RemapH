# ============================================================

# 🪟 RemapH V3

# 04_Platform.md

# ============================================================

## 🎯 Propósito

Este documento define la arquitectura de Platform.
Platform es la capa encargada de comunicar RemapH con el sistema operativo y el hardware.
Su función es traducir información entre el mundo interno del programa y los sistemas externos.
No contiene reglas de remapeo.
No toma decisiones.
No conoce la interfaz.

---

# 📑 Índice

1. Objetivo de Platform
2. Responsabilidad
3. Arquitectura general
4. Entrada física
5. Salida física
6. Interception
7. Tauri
8. Comunicación con Runtime
9. Reentrada
10. Reglas de diseño

---

# 🎯 1. Objetivo de Platform

Platform permite que RemapH interactúe con Windows sin contaminar las capas superiores.
Su función principal es:

```
Mundo externo
↓
Platform
↓
Información interna
```

y:

```
Decisión interna
↓
Platform
↓
Acción física
```

---

# 🏛️ 2. Responsabilidad

Platform contiene:
• Comunicación con Windows.
• Acceso a hardware.
• Captura física.
• Inyección física.
• Integración con Tauri.
• Traducción entre formatos externos e internos.

Platform no contiene:
• Reglas de remapeo.
• Configuración del usuario.
• Lógica de perfiles.
• Decisiones de ejecución.
• Componentes visuales.

---

# 🔄 3. Arquitectura general

La comunicación completa es:

```
Hardware
↓
Platform
↓
Runtime
↓
Platform
↓
Hardware
```

Platform funciona como intermediario.
Nunca como controlador del comportamiento.

---

# ⌨️ 4. Entrada física

La entrada física representa eventos provenientes del usuario.
Ejemplos:
• Teclado.
• Mouse.
• Dispositivos futuros.

Flujo:
```
Dispositivo físico
↓
Interception / Windows
↓
Platform
↓
Evento interno
```

La entrada debe convertirse a un formato común antes de llegar al Runtime.
El Runtime nunca debe depender del dispositivo original.

---

# ▶️ 5. Salida física

La salida física ejecuta acciones solicitadas por Runtime.
Ejemplos:

• Pulsar tecla.
• Soltar tecla.
• Mover mouse.
• Ejecutar multimedia.
• Dispositivos futuros.

---

Flujo:
```
Runtime
↓
Solicitud de acción
↓
Platform
↓
Hardware
```

---

# 🧩 6. Interception

Interception pertenece exclusivamente a Platform.
Es la capa encargada de trabajar con la librería de bajo nivel.
Responsabilidades:
• Inicialización.
• Recepción de eventos físicos.
• Reenvío.
• Inyección.
• Comunicación con dispositivos.

Interception no conoce:
• Remapeos.
• Perfiles.
• UI.
• Acciones del usuario.

---

# 🖥️ 7. Tauri

Tauri es el puente entre la aplicación y el entorno del sistema.
Responsabilidades:
• Crear la aplicación de escritorio.
• Exponer comandos.
• Comunicar UI con lógica nativa.
• Gestionar integración del sistema.

Tauri no debe contener reglas del remapeador.
Solo transporta información.

---

# 🔗 8. Comunicación con Runtime

La comunicación entre Platform y Runtime debe mantenerse simple.

```
Platform informa
↓
Runtime decide
↓
Platform ejecuta
```

---

Ejemplo:
Platform recibe:
```
Mouse Button X1 Down
```
Transforma a:
```
Evento interno
```
Runtime decide:
```
Enviar Ctrl+C
```
Platform ejecuta:
```
Inyección de teclado
```

---

# 🔁 9. Reentrada

Platform debe impedir que RemapH procese sus propios eventos generados.
Ejemplo:

```
Runtime genera Ctrl+C
↓
Platform inyecta tecla
↓
Sistema genera evento
↓
RemapH lo recibe
↓
Debe ignorarlo
```

El control de reentrada pertenece a Platform.
El Runtime nunca debe preocuparse por el origen físico del evento.

---

# 📌 10. Reglas de diseño

## Platform no decide
Platform ejecuta instrucciones.
No interpreta intención.

## Platform no conoce reglas
Nunca debe saber:
• qué es un perfil.
• qué es un remapeo.
• qué botón fue presionado.

## Platform traduce, no interpreta
Su trabajo es convertir formatos.

## Runtime decide, Platform ejecuta
Esta separación es una regla fundamental de RemapH.

---

# ✅ Resumen

Platform es la frontera entre RemapH y el sistema operativo.
Sus responsabilidades son:
• Escuchar hardware.
• Generar entradas internas.
• Ejecutar salidas físicas.
• Comunicar con Windows.
• Proteger la separación del resto del sistema.
Platform es el cuerpo de RemapH.
Runtime es su cerebro.