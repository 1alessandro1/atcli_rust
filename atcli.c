#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

// L'array dei terminatori per cui il programma interrompe la lettura dal modem.
const char *terminators[] = {
    "+CME ERROR:",
    "+CMS ERROR:",
    "BUSY\r\n",
    "ERROR\r\n",
    "NO ANSWER\r\n",
    "NO CARRIER\r\n",
    "NO DIALTONE\r\n",
    "OK\r\n",
    NULL
};

// Il disassemblato mostra che Compal usa buffer globali (unk_3410 e unk_2410)
// di dimensione 0x1000 (4096 bytes) invece che variabili locali.
char read_buffer[4096]; 
char copy_buffer[4096]; 

int send_at_command(const char *at_command) {
    // loc_8EC: Apre /dev/smd11 in modalità "r+b"
    FILE *stream = fopen("/dev/smd11", "r+b");
    if (stream == NULL) {
        // loc_9AE: Errore apertura
        fprintf(stderr, "fopen(%s) failed: %s\n", "/dev/smd11", strerror(errno));
        return -1;
    }

    // Invia il comando al modem
    int res = fputs(at_command, stream);
    if (res < 0) {
        // loc_9E0: Errore invio (fputs ha ritornato un valore < 0)
        fprintf(stderr, "failed to send '%s' to modem (res = %d)\n", at_command, res);
        goto chiudi;
    }

    // loc_92A: Inizio del ciclo di lettura
    while (1) {
        // Legge max 0x1000 (4096) bytes
        if (fgets(read_buffer, sizeof(read_buffer), stream) == NULL) {
            // loc_994: Raggiunto EOF o errore di lettura
            fprintf(stderr, "EOF from modem\n");
            break;
        }

        // Stampa la risposta riga per riga per l'utente (equivalente di stpcpy/strlen originale)
        printf("%s", read_buffer);

        // loc_95C: Ciclo interno che controlla se la riga contiene un terminatore
        int break_loop = 0;
        for (int i = 0; terminators[i] != NULL; i++) {
            size_t len = strlen(terminators[i]);
            // loc_96E: strncmp tra il buffer letto e il terminatore
            if (strncmp(terminators[i], read_buffer, len) == 0) {
                break_loop = 1;
                break;
            }
        }
        
        // Se ha trovato "OK", "ERROR", ecc... esce dal loop
        if (break_loop) {
            break;
        }
    }

chiudi:
    // loc_982: Chiusura dello stream
    if (fclose(stream) != 0) {
        // loc_9FC: Errore durante la chiusura
        fprintf(stderr, "closing modem failed: %s\n", strerror(errno));
        return -1;
    }

    return 0;
}

int main(int argc, char *argv[]) {
    // Controllo base degli argomenti (da sub_8B0 o simili, ma standardizzato)
    if (argc < 2) {
        printf("Reversed AT CLI for Qualcomm modems\n");
        printf("  Usage: %s [at command]\n", argv[0]);
        return 1;
    }

    // Il modem si aspetta \r\n, formatto l'argomento
    char command_with_crlf[1024];
    snprintf(command_with_crlf, sizeof(command_with_crlf), "%s\r\n", argv[1]);

    send_at_command(command_with_crlf);

    return 0;
}
