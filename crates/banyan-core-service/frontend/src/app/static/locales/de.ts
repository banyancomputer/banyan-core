export default {
    coponents: {
        account: {
            billing: {
                invoices: {
                    title: "Rechnungen",
                    emptyStateTitle: "Keine Zahlungs- oder Abrechnungsaktivität",
                    emptyStateDescription: "Rechnungen und Quittungen, die auf Ihrem Konto erstellt wurden, stehen hier zur Verfügung",
                    upgradeAccount: "Konto aktualisieren",
                    manageSubscriptions: "Abonnements verwalten",
                    invoice: {
                        title: "Rechnung",
                        summary: "Zusammenfassung",
                        to: "An",
                        from: "Von",
                        subscribedPlan: "Abonniertes Paket",
                        billingPeriod: "Abrechnungszeitraum",
                        items: "Artikel",
                        driveSrorage: "Speicher",
                        archiveStorage: "Archivspeicher",
                        dataEggress: "Daten-Ausgang",
                        payment: "Zahlung",
                        total: "Gesamt"
                    },
                    invoicesTable: {
                        date: "Datum",
                        status: "Status",
                        totalCost: "Gesamtkosten",
                        details: "Details",
                        viewInvoice: "Rechnung anzeigen"
                    },
                    nextBillingDate: {
                        title: "Nächstes Abrechnungsdatum",
                        onDemandStorage: "Bedarfsspeicher",
                        archivalStorage: "Archivspeicher",
                        dataEggress: "Daten-Ausgang",
                        totalCost: "Gesamtkosten",
                        upgradeAccount: "Konto aktualisieren",
                        manageSubscriptions: "Abonnements verwalten",
                    },
                }
            },
            manageKeys: {
                keyActions: {
                    rename: "Umbenennen",
                    removeKey: "Schlüssel entfernen",
                    lastKeyError: 'Der letzte Schlüssel kann nicht deaktiviert oder entfernt werden, ohne mindestens ein Backup.'
                },
                keyManagementTable: {
                    key: "Schlüssel",
                    device: "Gerät",
                    drive: "Laufwerk",
                    createdOn: "Erstellt am",
                    disable: "Deaktivieren",
                    enable: "Aktivieren",
                }
            },
            navigation: {
                title: "Kontoeinstellungen",
                profile: "Profil",
                manageAccessKeys: "Zugriffsschlüssel verwalten",
                billingAndPayment: "Abrechnung und Zahlung"
            },
            profile: {
                name: "Name",
                email: "E-Mail-Adresse",
                darkMode: "Dunkelmodus",
                language: "Sprache"
            },
        },
        bucket: {
            files: {
                bucketTable: {
                    name: "Name",
                    lastModified: "Zuletzt geändert",
                    fileSize: "Dateigröße",
                    moveToError: "Beim Verschieben Ihrer Datei ist ein Problem aufgetreten. Bitte versuchen Sie es erneut.",
                    tryAgain: "Erneut versuchen",
                    fileWasMoved: "Datei wurde verschoben",
                    uploadError: "Beim Hochladen ist ein Problem aufgetreten. Bitte versuchen Sie es erneut.",
                    fileActions: {
                        download: "Herunterladen",
                        moveTo: "Verschieben nach",
                        makeCopy: "Kopie erstellen",
                        viewFileVersions: "Dateiversionen anzeigen",
                        rename: "Umbenennen",
                        remove: "Entfernen",
                        shareFile: "Datei teilen",
                        yourFileIsSecure: "Ihre Datei ist sicher",
                        tryAgain: "Erneut versuchen",
                        downloading: "Herunterladen",
                        fileWasDownloaded: "Datei wurde heruntergeladen",
                        copyOf: "Kopie von",
                        wasCreated: "wurde erstellt",
                    },
                    folderActions: {
                        moveTo: "Verschieben nach",
                        rename: "Umbenennen",
                        remove: "Entfernen",
                        upload: "Hochladen",
                    },
                    folderRow: {
                        tryAgain: "Erneut versuchen",
                        fileWasMoved: "Datei wurde verschoben",
                        moveToError: "Beim Verschieben Ihrer Datei ist ein Problem aufgetreten. Bitte versuchen Sie es erneut.",
                        uploadError: "Beim Hochladen ist ein Problem aufgetreten. Bitte versuchen Sie es erneut.",
                        failedToLoadFiles: "Fehler beim Laden der Dateien"
                    }
                },
                emptyState: {
                    description: "Ziehen Sie Dateien hierher, um sie hochzuladen, oder verwenden Sie die Schaltfläche „Hochladen“",
                    buttonText: "Hochladen",
                    tryAgain: "Erneut versuchen",
                    uploadError: "Beim Hochladen ist ein Problem aufgetreten. Bitte versuchen Sie es erneut.",
                },
                header: {
                    files: "Dateien",
                    uploadButton: "Hochladen",
                    createFolderButton: "Ordner erstellen",
                    snapshotBannerTitle: "Archiv-Snapshots",
                    snapshotBannerSubtitle: "Dieses Laufwerk hat keine Snapshots",
                    snapshotBannerExplanation: "Was ist ein Snapshot",
                    snapshotBannerTooltip: "Archiv-Snapshots bieten einen Momentaufnahme des Dateiinhalts und sind nützlich für die Versionierung",
                    makeSnapshot: "Snapshot erstellen",
                },
            },
            snapshots: {
                title: "Snapshots",
                table: {
                    name: "Name",
                    date: "Datum",
                    size: "Größe",
                    state: "Status",
                    snapshotActions: {
                        rename: "Umbenennen",
                        restore: "Wiederherstellen"
                    }
                },
            }
        },
        home: {
            bucket: {
                hot: "Bedarfsorientiert",
                warm: "Bedarfsorientiert",
                cold: "Archiv",
                coldTooltip: "Archivspeicher ist für Dateien gedacht, die Sie sicher für längere Zeit speichern möchten und auf die Sie nicht häufig zugreifen möchten.",
                hotTooltip: "Bedarfsorientierter Speicher ist für Dateien gedacht, auf die Sie häufig zugreifen und auf die schnell zugreifen müssen.",
                warmTooltip: "Bedarfsorientierter Speicher ist für Dateien gedacht, auf die Sie häufig zugreifen und auf die schnell zugreifen müssen.",
                coldSnapshots: "Archiv-Snapshots",
                files: "Dateien",
                uploadError: "Beim Hochladen ist ein Problem aufgetreten. Bitte versuchen Sie es erneut",
                tryAgain: "Erneut versuchen",
            },
            emptyState: {
                title: "Erstellen Sie Ihr erstes Laufwerk, um mit dem Hochladen und Freigeben zu beginnen",
                newDriveButton: "Neues Laufwerk",
            },
        },
        common: {
            betaBanner: {
                title: "Banyan Storage ist noch in der Beta-Phase und nicht für die Produktion geeignet. Wir empfehlen, für alles, was auf Banyan gespeichert ist, eine Backup-Speichermethode zu verwenden. Wenn Sie Daten noch heute in Prod speichern müssen, wenden Sie sich an tim@banyan.computer"
            },
            bucketActions: {
                upload: "Hochladen",
                takeArchivalSnapshot: "Archiv-Snapshot erstellen",
                viewArchivalSnapshots: "Archiv-Snapshots anzeigen",
                viewDriveVersions: "Laufwerk versionen anzeigen",
                rename: "Umbenennen",
                createFolder: "Ordner erstellen",
                restoreCold: "Kalte Version wiederherstellen",
                deleteHotData: "Heiße Daten löschen",
                delete: "Löschen",
                purgeColdKeys: "Kalte Schlüssel löschen",
                unlock: "Entsperren",
                snapshotExplanation: "Archiv-Snapshots bieten einen Momentaufnahme des Dateiinhalts und sind nützlich für die Versionierung",
            },
            filePreview: {
                downloading: "Herunterladen",
                fileWasDownloaded: "Datei wurde heruntergeladen",
                tryAgain: "Erneut versuchen",
                shareFile: "Datei teilen",
                fileIsNotSupported: "Datei kann nicht für die Vorschau verwendet werden",
                actions: {
                    moveTo: "Verschieben nach",
                    rename: "Umbenennen",
                    remove: "Entfernen",
                }
            },
            folderSelect: {
                createFolder: "Ordner erstellen",
            },
            header: {
                upgrade: "Aktualisieren",
                helpControls: {
                    contactUs: "Kontaktieren Sie uns",
                },
                profileControls: {
                    settings: "Einstellungen",
                    logout: "Abmelden",
                    upgrade: "Aktualisieren",
                }
            },
            mobilePlaceholder: {
                title: "Sie sind bereit zu beginnen!",
                subtitle: "Um Banyan zu verwenden, verwenden Sie bitte den Desktop",
            },
            navigation: {
                uploadError: "Beim Hochladen ist ein Problem aufgetreten. Bitte versuchen Sie es erneut",
                tryAgain: "Erneut versuchen",
                allDrives: "Alle Laufwerke",
                lockedTooltip: {
                    youHaveNoAccess: "Ihre Daten sind sicher. Während der WNFS-Migration ist dieser Bucket gesperrt. Er wird in Kürze entsperrt. Kontaktieren Sie uns bei Fragen.",
                    requestAccess: "Zugriff anfordern",
                    here: "hier",
                }
            },
            storageUsage: {
                storage: "Speicher",
                used: "Verwendet",
                of: "von",
                upgradePlan: "Plan aktualisieren",
            },
            termsAndConditions: {
                accountTypeQuestion: "Wofür verwenden Sie Banyan?",
                agreeToTerms: "Ich akzeptiere die",
                termsOf: "Nutzungsbedingungen",
                and: "und",
                privacyPolicy: "Datenschutzbestimmungen",
                continue: "Fortsetzen",
            },
            uploadFileProgress: {
                uploading: "Hochladen",
                uploadFailed: "Upload fehlgeschlagen"
            },
            modal: {
                approveBucketAccess: {
                    title: "Zugriff genehmigen",
                    subtitle: "Sind Sie sicher, dass Sie den Zugriff genehmigen möchten ",
                    tryAgain: "Erneut versuchen",
                    cancel: "Abbrechen",
                    approveAccess: "Zugriff genehmigen",
                },
                createBucket: {
                    createNewDrive: "Neues Laufwerk erstellen",
                    driveName: "Laufwerkname",
                    enterNewName: "Neuen Namen eingeben",
                    cancel: "Abbrechen",
                    create: "Erstellen",
                    creationError: "Beim Erstellen Ihres Laufwerks ist ein Problem aufgetreten. Bitte versuchen Sie es erneut.",
                    tryAgain: "Erneut versuchen",
                },
                createFolder: {
                    title: "Ordner erstellen",
                    folderName: "Ordnername",
                    enterNewName: "Neuen Namen eingeben",
                    cancel: "Abbrechen",
                    create: "Erstellen",
                    creationError: "Beim Erstellen Ihres Ordners ist ein Problem aufgetreten. Bitte versuchen Sie es erneut.",
                    tryAgain: "Erneut versuchen",
                },
                createSecretKey: {
                    title: "Geheimen Schlüssel erstellen",
                    subtitle: "Ihr geheimer Schlüssel ist der Weg, um Ihr Konto zu entsperren; erstellen Sie einen geheimen Schlüssel, den Sie sich merken können und halten Sie ihn sicher auf – niemand sonst hat Zugriff darauf, nicht einmal Banyan, also stellen Sie sicher, dass Sie sich daran erinnern können, um Ihr Konto zu entsperren.",
                    secretKey: "Geheimer Schlüssel",
                    enterSecretKey: "Geheimen Schlüssel eingeben",
                    keyRequirements: "Der Schlüssel muss mindestens 8 Zeichen lang sein",
                    confirmSecretKey: "Geheimen Schlüssel bestätigen",
                    passphraseNotMatch: "Die Passwörter stimmen nicht überein, bitte versuchen Sie es erneut",
                    confirm: "Bestätigen",
                    creationError: "Fehler beim Initialisieren des Schlüsselspeichers"
                },
                deleteBucket: {
                    title: "Laufwerk löschen",
                    subtitle: "Sind Sie sicher, dass Sie löschen möchten",
                    filesWillBeDeletedPermanently: "Dateien werden dauerhaft gelöscht",
                    delete: "Löschen",
                    cancel: "Abbrechen",
                    drive: "Laufwerk",
                    wasDeleted: "wurde gelöscht",
                    deletionError: "Beim Löschen ist ein Problem aufgetreten. Bitte versuchen Sie es erneut",
                    tryAgain: "Erneut versuchen",
                },
                deleteFile: {
                    title: "Datei entfernen",
                    wantToMove: "Möchten Sie entfernen",
                    filesWillBeMoved: "Datei wird dauerhaft gelöscht",
                    delete: "Löschen",
                    cancel: "Abbrechen",
                    file: "Datei",
                    wasDeleted: "wurde gelöscht",
                    deletionError: "Beim Löschen ist ein Problem aufgetreten. Bitte versuchen Sie es erneut",
                    tryAgain: "Erneut versuchen",
                },
                enterSecretKey: {
                    title: "Geheimen Schlüssel eingeben",
                    subtitle: "Geben Sie den geheimen Schlüssel in das Textfeld ein",
                    secretKey: "Geheimer Schlüssel",
                    enterSecretKey: "Geheimen Schlüssel eingeben",
                    keyRequirements: "Der Schlüssel muss mindestens 8 Zeichen lang sein",
                    confirm: "Bestätigen",
                    sectretKeyError: "Falscher geheimer Schlüssel",
                },
                hardStorageLimit: {
                    title: "Ihr Speicher ist erschöpft",
                    subtitle: "Aktualisieren Sie Ihr Konto, um Dateien hochzuladen und zu synchronisieren.",
                    cancel: "Abbrechen",
                    upgradePlan: "Plan aktualisieren",
                },
                moteTo: {
                    title: "Verschieben nach",
                    subtitle: "Bitte wählen Sie aus, wohin Sie Ihre Datei verschieben möchten",
                    folder: "Ordner",
                    cancel: "Abbrechen",
                    moveTo: "Verschieben nach",
                    fileWasMoved: "Datei wurde verschoben",
                    moveToError: "Beim Verschieben Ihrer Datei ist ein Problem aufgetreten. Bitte versuchen Sie es erneut",
                    tryAgain: "Erneut versuchen",
                },
                removeBucketAccess: {
                    title: "Zugriff entfernen",
                    subtitle: "Das Entfernen des Zugriffs bedeutet, dass Sie in Zukunft keinen Zugriff auf diese Daten haben. Alle zuvor heruntergeladenen Daten sind für Sie weiterhin zugänglich, werden jedoch nicht mehr synchronisiert",
                    cancel: "Abbrechen",
                    removeAccess: "Zugriff entfernen"
                },
                renameBucket: {
                    title: "Laufwerk umbenennen",
                    enterNewName: "Neuen Namen eingeben",
                    cancel: "Abbrechen",
                    save: "Speichern",
                    drive: "Laufwerk",
                    wasRenamed: "wurde umbenannt",
                    editError: "Beim Bearbeiten ist ein Problem aufgetreten. Bitte versuchen Sie es erneut",
                    tryAgain: "Erneut versuchen",
                },
                renameFile: {
                    title: "Datei umbenennen",
                    fileName: "Dateiname",
                    enterNewName: "Neuen Namen eingeben",
                    cancel: "Abbrechen",
                    save: "Speichern",
                    fileWasRenamed: "Datei wurde umbenannt",
                    editError: "Beim Bearbeiten ist ein Problem aufgetreten. Bitte versuchen Sie es erneut",
                    tryAgain: "Erneut versuchen",
                },
                renameAccessKey: {
                    title: "Schlüssel umbenennen",
                    keyName: "Schlüsselname",
                    enterNewName: "Neuen Namen eingeben",
                    cancel: "Abbrechen",
                    save: "Speichern",
                    keyWasRenamed: "Der Schlüssel wurde umbenannt",
                    editError: "Es gab ein Problem mit Ihrer Bearbeitung. Bitte versuchen Sie es erneut",
                    tryAgain: "Erneut versuchen"
                },
                renameSnapshot: {
                    title: "Snapshot umbenennen",
                    snapshotName: "Snapshot-Name",
                    enterNewName: "Neuen Namen eingeben",
                    cancel: "Abbrechen",
                    save: "Speichern",
                    snapshotWasRenamed: "Snapshot wurde umbenannt",
                    editError: "Es gab ein Problem mit Ihrer Bearbeitung. Bitte versuchen Sie es erneut",
                    tryAgain: "Erneut versuchen"
                },
                requestBucketAccess: {
                    title: "Zugriff anfordern",
                    subtitle: "Möchten Sie Zugriff anfordern?",
                    cancel: "Abbrechen",
                    requestAccess: "Zugriff anfordern",
                },
                shareFile: {
                    title: "Teilen",
                    cancel: "Abbrechen",
                    copyLink: "Link kopieren",
                    linkWasCopied: "Link wurde kopiert",
                },
                subscriptionPlan: {
                    title: "Wählen Sie einen Plan, der zu Ihnen passt",
                    subtitle: "Erhalten Sie den Speicher, die Sicherheit und die Privatsphäre, die Sie verdienen, zu einem erschwinglichen Preis",
                    hotStorage: "Bedarfsspeicher",
                    hotReplications: "Bedarfsreplikationen",
                    freeEgress: "Kostenloser Datenabgang",
                    archivalSnapshots: "Archiv-Snapshots",
                    litePlanDescription: "Kostenlos, aber mit eingeschränkten Funktionen",
                    currentPlan: "Aktueller Plan",
                    upgradeTo: "Aktualisieren auf",
                    needCustomPlan: "Benötigen Sie einen individuelleren Plan",
                    contactSales: "Vertrieb kontaktieren",
                },
                takeSnapshot: {
                    title: "Archiv-Snapshot erstellen",
                    subtitle: "Möchten Sie einen Archiv-Snapshot erstellen",
                    cancel: "Abbrechen",
                    takeArchivalSnapshot: "Archiv-Snapshot erstellen",
                    snapshotWasTaken: "Archiv-Snapshot wurde erstellt",
                    snapshotError: "Beim Snapshot ist ein Problem aufgetreten. Bitte versuchen Sie es erneut.",
                    tryAgain: "Erneut versuchen",
                },
                termsAndConditions: {
                    title: "Nutzungsbedingungen & Datenschutzbestimmungen",
                    decline: "Ablehnen",
                    acceptTermsAndService: "Ich habe die Nutzungsbedingungen von Banyan gelesen und akzeptiert",
                },
                uploadFile: {
                    title: "Dateien hochladen",
                    subtitle: "Wählen Sie Dateien zum Hochladen von Ihrem Gerät aus oder verwenden Sie Drag & Drop",
                    selectDrive: "Laufwerk auswählen",
                    createNewDrive: "Neues Laufwerk erstellen",
                    selectFolder: "Ordner auswählen",
                    clickToUpload: "Klicken Sie zum Hochladen",
                    orDragAndDrop: "oder ziehen Sie Dateien hierher",
                    cancel: "Abbrechen",
                    upload: "Hochladen",
                    uploadError: "Beim Hochladen ist ein Problem aufgetreten. Bitte versuchen Sie es erneut",
                    tryAgain: "Erneut versuchen",
                }
            }
        },
    },
    pages: {
        home: {
            allDrives: "Alle Laufwerke",
            upload: "Hochladen",
            newDrive: "Neues Laufwerk",
        },
        registerDevice: {
            title: "Zugriff genehmigen",
            wantToApproveAccess: "Sind Sie sicher, dass Sie den Zugriff genehmigen möchten",
            cancel: "Abbrechen",
            approveAccess: "Zugriff genehmigen",
        },
        trash: {
            trash: "Papierkorb",
            trashIsFull: "Papierkorb ist voll",
            emptyTrash: "Papierkorb leeren",
            trashIsEmpty: "Papierkorb ist leer",
            clickToEmptyTrash: "Klicken Sie zum Leeren des Papierkorbs",
        }
    },
    contexts: {
        fileUpload: {
            softStorageLimit: "Sie nähern sich dem Limit Ihres Speicherplans, aktualisieren Sie für mehr Platz",
            hardStorageLimit: "Ihre Speicheranforderung übersteigt die Speicherkapazität, bitte",
            seePricingPage: "Besuchen Sie unsere Preisgestaltungsseite",
            contactSales: "Vertrieb kontaktieren",
            fileSizeExceeded: "Dateigröße überschritten. Bitte versuchen Sie es erneut mit einer Datei kleiner als 100 MB oder verwenden Sie die Banyan-Befehlszeilenschnittstelle."
        },
        tomb: {
            folderAlreadyExists: 'Ordnername muss eindeutig sein - bitte geben Sie einen eindeutigen Namen ein',
            driveAlreadyExists: 'Laufwerksname muss eindeutig sein - bitte geben Sie einen eindeutigen Namen ein'
        }
    }
};