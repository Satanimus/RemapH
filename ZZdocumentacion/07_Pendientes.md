# ============================================================

# 📌 RemapH V3

# 07_Pendientes.md

# ============================================================

## 🎯 Propósito

Este documento registra funcionalidades, mejoras e investigaciones pendientes del proyecto.
No contiene decisiones definitivas de arquitectura.
No sustituye los documentos oficiales de cada capa.
Su objetivo es mantener una lista clara de trabajo futuro.

---

# 📑 Índice

1. Pendientes de desarrollo
2. Mejoras de arquitectura
3. Sistemas futuros
4. Documentación futura
5. Ideas descartadas o en evaluación

---

# 🔨 1. Pendientes de desarrollo

## 🔵 Sistema de entrada unificada

Crear una representación interna única para todos los dispositivos.
Debe permitir:
• Teclado.
• Mouse.
• Multimedia.
• Dispositivos futuros.
La capa superior nunca debe depender del hardware específico.

---

## 🔵 Captura avanzada

Completar sistema de captura:
• Combinaciones de teclas.
• Modificadores.
• Mouse.
• Secuencias.
• Estados temporales.

---

## 🔵 Sistema de perfiles

Implementar gestión completa de perfiles.
Flujo esperado:
```
Guardar perfil actual
↓
Desactivar perfil actual
↓
Limpiar interfaz
↓
Cargar nuevo perfil
↓
Reconstruir configuración
```

El cambio de perfil es una acción global.
No debe modelarse como evento de UI.

---

## 🔵 Preferencias globales

Agregar configuración general:
• Tiempos.
• Comportamientos por defecto.
• Apariencia.
• Opciones del sistema.

---

## 🔵 Editor de macros

Crear editor visual para secuencias complejas.
Debe permanecer separado de la configuración básica de una fila.

---

## 🔵 Ventanas especializadas

Crear ventanas independientes cuando la complejidad lo justifique.
Ejemplos:
• Multimedia avanzada.
• Programas.
• Coordenadas.
• Macros.
• Joystick.

---

# 🏗️ 2. Mejoras de arquitectura

## 🧠 Sistema ADR

Crear un sistema de registro de decisiones arquitectónicas.
Cada decisión importante debe documentar:
• Fecha.
• Problema existente.
• Opciones consideradas.
• Decisión tomada.
• Motivo.
• Consecuencias.

---

## Motivo
Las decisiones de arquitectura no deben depender únicamente de memoria personal.
El proyecto debe poder retomarse meses o años después manteniendo su coherencia.

---

# 🧩 3. Sistemas futuros

## 🔵 Joystick

Agregar soporte para dispositivos adicionales.
Debe seguir la misma arquitectura:
```
Platform
↓
Evento interno
↓
Runtime
↓
Acción
```

---

## 🔵 Sistema de conflictos

Detectar:
• Dos remapeos usando el mismo trigger.
• Acciones incompatibles.
• Configuraciones ambiguas.

---

## 🔵 Importación y exportación

Permitir compartir configuraciones.
Debe mantenerse independiente del formato interno compilado.

---

# 📚 4. Documentación futura

Mantener actualizados:
• Índice maestro.
• Documentos de arquitectura.
• ADR.
• Notas de versiones.

Toda decisión importante debe documentarse antes de convertirse en una dependencia del proyecto.

---

# 💭 5. Ideas descartadas o en evaluación

Esta sección almacena ideas que todavía no tienen una decisión definitiva.
Ejemplos:
• Nuevas formas de interfaz.
• Cambios de organización.
• Nuevas tecnologías.

Una idea pendiente no debe convertirse automáticamente en una implementación.
Primero debe evaluarse su impacto arquitectónico.

---

# ✅ Resumen

Los pendientes representan el futuro del proyecto.
Sus reglas principales:
• Separar ideas de decisiones.
• Registrar cambios importantes.
• Mantener la arquitectura antes de agregar funciones.
• Resolver complejidad con nuevas capas cuando corresponda.
• Evitar crecimiento desordenado.