Motor captura / reconocimiento trigger

✅ Renombres iniciales completados
✅ Separación Runtime / Captura / Cache
✅ EventoTrigger creado
✅ InputEvent definido como evento físico
✅ Estado global de perfil separado
✅ Decisión: tiempo pertenece al evento físico
❌ BufferEventos eliminado como módulo independiente
✅ Agregar Instante a InputEvent
⬜ Crear AnalizadorTrigger definitivo
⬜ Mover lógica de Simple/Doble/Mantenido al analizador
⬜ Implementar timeline interna del analizador
⬜ Implementar modificadores activos
⬜ Conectar AnalizadorTrigger → EventoTrigger
⬜ Revisar Runtime para consumir solamente EventoTrigger

ETAPA 3 — Analizador de Trigger

⬜ Implementar el análisis completo del trigger.

Detectar:

⬜ Simple
⬜ Doble
⬜ Mantenido

⬜ Resolver modificadores.
⬜ Identificar el gatillo.
⬜ Construir el InputEvent lógico que recibirá el Runtime.

ETAPA 4 — Integración

Estado actual del flujo:

Capturador
↓
InputEvent (con instante)
↓
AnalizadorTrigger
↓
EventoTrigger
↓
Runtime

Pendiente:

⬜ Conectar Capturador → AnalizadorTrigger
⬜ Eliminar referencias antiguas a BufferEventos
⬜ Asegurar que Runtime no interprete tiempos ni triggers
⬜ Runtime consume solamente eventos lógicos completos

ETAPA 5 — Validación

⬜ Probar teclado.
⬜ Probar mouse.
⬜ Probar rueda.
⬜ Probar modificadores.
⬜ Probar doble toque.
⬜ Probar mantenido.
⬜ Confirmar que Runtime recibe únicamente eventos lógicos completos.

////////////////////////////////////////////

📌 Ideas pendientes (no hacer todavía)

Estas no forman parte del objetivo actual, pero ya quedaron detectadas.

Arquitectura
⬜ Evaluar si AnalizadorTrigger debe convertirse únicamente en el analizador lógico mientras BufferEventos decide cuándo una secuencia está completa.
⬜ Elaborar un diccionario oficial de términos del proyecto para evitar ambigüedades futuras.
Rendimiento
⬜ Revisar el tamaño óptimo del BufferEventos.
⬜ Optimizar la reutilización de memoria del buffer.
Extensiones futuras
⬜ Integrar joystick utilizando la misma arquitectura de captura.
⬜ Evaluar soporte para nuevos tipos de triggers si fueran necesarios.

------ Recordar limpiar comandos.rs
------ igual captura.rs (eliminar)

------Documentar oficialmente el flujo del motor, igual que hicimos con el flujo Perfil → Compilador → Cache → Runtime.

Algo como:
Windows / Interception
│
▼
CapturadorTrigger
│
▼
BufferEventos
│
▼
AnalizadorTrigger
│
▼
ProcesadorEventos
│
▼
Runtime
│
▼
Emisor

Ese diagrama será muy útil cuando dentro de unos meses tengamos que volver al código.

----- Cuando terminemos el motor, podríamos hacer que BufferEventos tenga un modo de depuración que imprima la línea temporal completa. Algo como:

00.000 Ctrl Down
00.120 A Down
00.145 A Up
00.310 Ctrl Up

Sería una herramienta excelente para depurar problemas de triggers sin tocar el resto del sistema.
