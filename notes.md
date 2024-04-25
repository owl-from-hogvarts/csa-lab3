
# Language

Ассемблер

## Требования

синтаксис ассемблера. Необходима поддержка label-ов.

# Архитектура

Аккумуляторная

- система команд должна быть выстроена вокруг аккумулятора.
- Инструкции -- изменяют значение, хранимое в аккумуляторе.
- Ввод-вывод осуществляется через аккумулятор

# Организация памяти

фон неймана (команды и данные хранятся в одной памяти)

# Microcode

- Моделирование должно выполняться с точностью до такта.
- Микрокод должен быть сохранён в отдельной памяти для микропрограмм.
- Модель процессора должна исполнять микрокод.
- Точность модели -- потактовая (tick).

# Хранение кода

В json. Читается из файла. Процессор имеет магический загрузчик. Загрузчик интерпретирует специальные команды. Может быть реализовано с помощью секций

# Stream

Ввод-вывод осуществляется как поток токенов. Есть в примере. Логика работы:

- при старте модели у вас есть буфер, в котором представлены все данные ввода (['h', 'e', 'l', 'l', 'o']);
- при обращении к вводу (выполнение инструкции) модель процессора получает "токен" (символ) информации;
- если в буфере кончились данные и встретилась команда IN, то останавливать моделирование
- вывод данных реализуется аналогично, по выполнении команд в буфер вывода добавляется ещё один символ;
- по окончании моделирования показать все выведенные данные;
- логика работы с буфером реализуется в рамках модели на Python

Буфер ввода, буфер вывода.

# Ввода вывод

> port-mapped (специальные инструкции для ввода-вывода)
> адресация портов ввода-вывода должна присутствовать

Данные для ввода беруться из файла. Данные для ввода в файле должны преобразовываться в pstr "магически" при старте симуляции.
Вывод происходит на экран.

Инструкции IN, OUT. 1 байт на адресацию портов

# Строки

Pascal based: length prefixed

# Алгоритмы

> prob2

- Even Fibonacci numbers
- hello world
- cat
- hello_user_name
