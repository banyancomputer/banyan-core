export default {
    coponents: {
        account: {
            billing: {
                invoices: {
                    title: "Factures",
                    emptyStateTitle: "Aucune activité de paiement ou de facturation",
                    emptyStateDescription: "Les factures et reçus émis sur votre compte seront disponibles ici",
                    upgradeAccount: "Mettre à niveau le compte",
                    manageSubscriptions: "Gérer les abonnements",
                    invoice: {
                        title: "Facture",
                        summary: "Résumé",
                        to: "À",
                        from: "De",
                        subscribedPlan: "Plan abonné",
                        billingPeriod: "Période de facturation",
                        items: "Articles",
                        driveSrorage: "Stockage du lecteur",
                        archiveStorage: "Stockage d'archive",
                        dataEggress: "Egress de données",
                        payment: "Paiement",
                        total: "Total"
                    },
                    invoicesTable: {
                        date: "Date",
                        status: "Statut",
                        totalCost: "Coût total",
                        details: "Détails",
                        viewInvoice: "Voir la facture"
                    },
                    nextBillingDate: {
                        title: "Date de facturation suivante",
                        onDemandStorage: "Stockage à la demande",
                        archivalStorage: "Stockage d'archives",
                        dataEggress: "Egress de données",
                        totalCost: "Coût total",
                        upgradeAccount: "Mettre à niveau le compte",
                        manageSubscriptions: "Gérer les abonnements",
                    },
                }
            },
            manageKeys: {
                keyActions: {
                    removeAccess: "Supprimer l'accès",
                    approveAccess: "Approuver l'accès",
                },
                keyManagementTable: {
                    device: "Dispositif",
                    client: "Client",
                    fingerprint: "Empreinte digitale",
                    status: "Statut",
                    approved: "Approuvé",
                    noAccess: "Pas d'accès",
                }
            },
            navigation: {
                title: "Paramètres du compte",
                profile: "Profil",
                manageAccessKeys: "Gérer les clés d'accès",
                billingAndPayment: "Facturation et paiement"
            },
            profile: {
                title: "Profil",
                language: "Langue",
                chooseYourLanguage: "Choisissez votre langue"
            },
        },
        bucket: {
            bucketTable: {
                name: "Nom",
                lastModified: "Dernière modification",
                fileSize: "Taille du fichier",
                moveToError: "Un problème est survenu lors du déplacement de votre fichier. Veuillez réessayer.",
                tryAgain: "Réessayer",
                fileWasMoved: "Le fichier a été déplacé",
                uploadError: "Un problème est survenu avec le téléchargement. Veuillez réessayer.",
                fileActions: {
                    download: "Télécharger",
                    moveTo: "Déplacer vers",
                    makeCopy: "Faire une copie",
                    viewFileVersions: "Voir les versions du fichier",
                    rename: "Renommer",
                    remove: "Supprimer",
                    shareFile: "Partager le fichier",
                    yourFileIsSecure: "Votre fichier est sécurisé",
                    tryAgain: "Réessayer",
                    downloading: "Téléchargement",
                    fileWasDownloaded: "Le fichier a été téléchargé",
                    copyOf: "Copie de",
                    wasCreated: "a été créé",
                },
                folderActions: {
                    moveTo: "Déplacer vers",
                    rename: "Renommer",
                    remove: "Supprimer",
                    upload: "Télécharger",
                },
                folderRow: {
                    tryAgain: "Réessayer",
                    fileWasMoved: "Le fichier a été déplacé",
                    moveToError: "Un problème est survenu lors du déplacement de votre fichier. Veuillez réessayer.",
                    uploadError: "Un problème est survenu avec le téléchargement. Veuillez réessayer.",
                    failedToLoadFiles: "Impossible de charger les fichiers"
                }
            },
            emptyState: {
                description: "Faites glisser-déposer des fichiers ici pour les téléverser, ou utilisez le bouton 'Téléverser'",
                buttonText: "Téléverser",
                tryAgain: "Réessayer",
                uploadError: "Un problème est survenu avec le téléchargement. Veuillez réessayer.",
            },
            header: {
                files: "Fichiers",
                uploadButton: "Téléverser",
                createFolderButton: "Créer un dossier",
                snapshotBannerTitle: "Instantanés d'archives",
                snapshotBannerSubtitle: "Ce lecteur n'a pas d'instantanés",
                snapshotBannerExplanation: "Qu'est-ce qu'un instantané",
                snapshotBannerTooltip: "Les instantanés d'archives offrent un aperçu à un moment donné du fichier et sont utiles pour la version",
                makeSnapshot: "Faire un instantané",
            }
        },
        home: {
            bucket: {
                hot: "À la demande",
                warm: "À la demande",
                cold: "Archives",
                coldTooltip: "Le stockage d'archives est destiné aux fichiers que vous souhaitez stocker en toute sécurité pendant longtemps et que vous ne prévoyez pas d'accéder très souvent.",
                hotTooltip: "Le stockage à la demande est destiné aux fichiers auxquels vous prévoyez d'accéder fréquemment et qui nécessitent un accès rapide.",
                warmTooltip: "Le stockage à la demande est destiné aux fichiers auxquels vous prévoyez d'accéder fréquemment et qui nécessitent un accès rapide.",
                coldSnapshots: "Instantanés d'archives",
                files: "Fichiers",
                uploadError: "Un problème est survenu avec le téléchargement. Veuillez réessayer",
                tryAgain: "Réessayer",
            },
            emptyState: {
                title: "Créez votre premier lecteur pour commencer à téléverser et à partager",
                newDriveButton: "Nouveau lecteur",
            },
        },
        common: {
            betaBanner: {
                title: "Banyan Storage est encore en version bêta et n'est pas prêt pour la production. Nous vous recommandons d'utiliser une méthode de stockage de sauvegarde pour tout ce qui est stocké sur Banyan pour le moment. Si vous devez stocker des données en Prod aujourd'hui, contactez tim@banyan.computer"
            },
            bucketActions: {
                upload: "Téléverser",
                takeArchivalSnapshot: "Prendre un instantané d'archive",
                viewArchivalSnapshots: "Voir les instantanés d'archives",
                viewDriveVersions: "Voir les versions du lecteur",
                rename: "Renommer",
                createFolder: "Créer un dossier",
                restoreCold: "Restaurer la version froide",
                deleteHotData: "Supprimer les données chaudes",
                delete: "Supprimer",
                purgeColdKeys: "Purger les clés froides",
                unlock: "Déverrouiller",
                snapshotExplanation: "Les instantanés d'archives offrent un aperçu à un moment donné du fichier et sont utiles pour la version",
            },
            filePreview: {
                downloading: "Téléchargement",
                fileWasDownloaded: "Le fichier a été téléchargé",
                tryAgain: "Réessayer",
                shareFile: "Partager le fichier",
                fileIsNotSupported: "Le fichier n'est pas pris en charge pour l'aperçu",
                actions: {
                    moveTo: "Déplacer vers",
                    rename: "Renommer",
                    remove: "Supprimer",
                }
            },
            folderSelect: {
                createFolder: "Créer un dossier",
            },
            header: {
                upgrade: "Mettre à niveau",
                helpControls: {
                    contactUs: "Nous contacter",
                },
                profileControls: {
                    settings: "Paramètres",
                    logout: "Déconnexion",
                    upgrade: "Mettre à niveau",
                }
            },
            mobilePlaceholder: {
                title: "Vous êtes prêt à commencer !",
                subtitle: "Pour commencer à utiliser Banyan, veuillez utiliser un ordinateur de bureau",
            },
            navigation: {
                uploadError: "Un problème est survenu avec le téléchargement. Veuillez réessayer",
                tryAgain: "Réessayer",
                allDrives: "Tous les lecteurs",
                lockedTooltip: {
                    youHaveNoAccess: "Vous n'avez pas accès à ce lecteur",
                    requestAccess: "Demander l'accès",
                    here: "ici",
                }
            },
            storageUsage: {
                storage: "Stockage",
                used: "Utilisé",
                of: "de",
                upgradePlan: "Plan de mise à niveau",
            },
            termsAndConditions: {
                accountTypeQuestion: "Pour quoi utilisez-vous Banyan ?",
                agreeToTerms: "J'accepte les",
                termsOf: "conditions d'utilisation",
                and: "et",
                privacyPolicy: "politique de confidentialité",
                continue: "Continuer",
            },
            uploadFileProgress: {
                uploading: "Téléchargement",
                uploadFailed: "Échec du téléchargement"
            },
            modal: {
                approveBucketAccess: {
                    title: "Approuver l'accès",
                    subtitle: "Êtes-vous sûr de vouloir approuver l'accès ",
                    tryAgain: "Réessayer",
                    cancel: "Annuler",
                    approveAccess: "Approuver l'accès",
                },
                bucketSnapshots: {
                    title: "Voir les instantanés d'archives",
                    subtitle: "Accédez et examinez les versions précédentes",
                    tryAgain: "Réessayer",
                    close: "Fermer",
                },
                createBucket: {
                    createNewDrive: "Créer un nouveau lecteur",
                    driveName: "Nom du lecteur",
                    enterNewName: "Entrez un nouveau nom",
                    cancel: "Annuler",
                    create: "Créer",
                    creationError: "Un problème est survenu lors de la création de votre lecteur. Veuillez réessayer.",
                    tryAgain: "Réessayer",
                },
                createFolder: {
                    title: "Créer un dossier",
                    folderName: "Nom du dossier",
                    enterNewName: "Entrez un nouveau nom",
                    cancel: "Annuler",
                    create: "Créer",
                    creationError: "Un problème est survenu lors de la création de votre dossier. Veuillez réessayer.",
                    tryAgain: "Réessayer",
                },
                createSecretKey: {
                    title: "Créer une clé secrète",
                    subtitle: "Votre clé secrète est le moyen de déverrouiller votre compte ; créez une clé secrète que vous pouvez mémoriser et gardez-la en sécurité – personne d'autre n'y a accès, pas même Banyan, alors assurez-vous de pouvoir vous en souvenir pour déverrouiller votre compte.",
                    secretKey: "Clé secrète",
                    enterSecretKey: "Entrez la clé secrète",
                    keyRequirements: "La clé doit comporter au moins 8 caractères",
                    confirmSecretKey: "Confirmer la clé secrète",
                    passphraseNotMatch: "Les phrases secrètes ne correspondent pas, veuillez réessayer",
                    confirm: "Confirmer",
                    creationError: "Impossible d'initialiser le magasin de clés"
                },
                deleteBucket: {
                    title: "Supprimer le lecteur",
                    subtitle: "Êtes-vous sûr de vouloir supprimer",
                    filesWillBeDeletedPermanently: "Les fichiers seront supprimés de façon permanente",
                    delete: "Supprimer",
                    cancel: "Annuler",
                    drive: "Lecteur",
                    wasDeleted: "a été supprimé",
                    deletionError: "Un problème est survenu avec la suppression. Veuillez réessayer",
                    tryAgain: "Réessayer",
                },
                deleteFile: {
                    title: "Supprimer le fichier",
                    wantToMove: "Voulez-vous supprimer",
                    filesWillBeMoved: "Le fichier sera supprimé de façon permanente",
                    delete: "Supprimer",
                    cancel: "Annuler",
                    file: "Fichier",
                    wasDeleted: "a été supprimé",
                    deletionError: "Un problème est survenu avec la suppression. Veuillez réessayer",
                    tryAgain: "Réessayer",
                },
                enterSecretKey: {
                    title: "Entrer la clé secrète",
                    subtitle: "Entrez la clé secrète dans le champ de texte",
                    secretKey: "Clé secrète",
                    enterSecretKey: "Entrez la clé secrète",
                    keyRequirements: "La clé doit comporter au moins 8 caractères",
                    confirm: "Confirmer",
                    sectretKeyError: "Clé secrète incorrecte",
                },
                hardStorageLimit: {
                    title: "Vous êtes à court de stockage",
                    subtitle: "Mettez à niveau votre compte pour téléverser et synchroniser des fichiers.",
                    cancel: "Annuler",
                    upgradePlan: "Plan de mise à niveau",
                },
                moteTo: {
                    title: "Déplacer vers",
                    subtitle: "Veuillez sélectionner où vous souhaitez déplacer votre fichier",
                    folder: "Dossier",
                    cancel: "Annuler",
                    moveTo: "Déplacer vers",
                    fileWasMoved: "Le fichier a été déplacé",
                    moveToError: "Un problème est survenu lors du déplacement de votre fichier. Veuillez réessayer",
                    tryAgain: "Réessayer",
                },
                removeBucketAccess: {
                    title: "Supprimer l'accès",
                    subtitle: "Supprimer l'accès signifie que vous n'aurez plus accès à ces données à l'avenir, toutes les données précédemment téléchargées sont toujours accessibles pour vous mais ne seront plus synchronisées",
                    cancel: "Annuler",
                    removeAccess: "Supprimer l'accès"
                },
                renameBucket: {
                    title: "Renommer le lecteur",
                    enterNewName: "Entrez un nouveau nom",
                    cancel: "Annuler",
                    save: "Enregistrer",
                    drive: "Lecteur",
                    wasRenamed: "a été renommé",
                    editError: "Un problème est survenu avec votre modification. Veuillez réessayer",
                    tryAgain: "Réessayer",
                },
                renameFile: {
                    title: "Renommer le fichier",
                    fileName: "Nom du fichier",
                    enterNewName: "Entrez un nouveau nom",
                    cancel: "Annuler",
                    save: "Enregistrer",
                    fileWasRenamed: "Le fichier a été renommé",
                    editError: "Un problème est survenu avec votre modification. Veuillez réessayer",
                    tryAgain: "Réessayer",
                },
                requestBucketAccess: {
                    title: "Demander l'accès",
                    subtitle: "Voulez-vous demander l'accès ?",
                    cancel: "Annuler",
                    requestAccess: "Demander l'accès",
                },
                shareFile: {
                    title: "Partager",
                    cancel: "Annuler",
                    copyLink: "Copier le lien",
                    linkWasCopied: "Le lien a été copié",
                },
                subscriptionPlan: {
                    title: "Choisissez un plan qui vous convient",
                    subtitle: "Obtenez le stockage, la sécurité et la confidentialité que vous méritez, à un prix abordable",
                    hotStorage: "Stockage à chaud",
                    hotReplications: "Réplications à chaud",
                    freeEgress: "Egress gratuit",
                    archivalSnapshots: "Instantanés d'archives",
                    litePlanDescription: "Gratuit, mais avec des fonctionnalités limitées",
                    currentPlan: "Plan actuel",
                    upgradeTo: "Mettre à niveau vers",
                    needCustomPlan: "Besoin d'un plan plus personnalisé",
                    contactSales: "Contacter les ventes",
                },
                takeSnapshot: {
                    title: "Prendre un instantané d'archive",
                    subtitle: "Voulez-vous prendre un instantané d'archive",
                    cancel: "Annuler",
                    takeArchivalSnapshot: "Prendre un instantané d'archive",
                    snapshotWasTaken: "L'instantané d'archive a été pris",
                    snapshotError: "Un problème est survenu avec votre instantané. Veuillez réessayer.",
                    tryAgain: "Réessayer",
                },
                termsAndConditions: {
                    title: "Conditions générales de vente et Politique de confidentialité",
                    decline: "Refuser",
                    acceptTermsAndService: "J'ai lu et j'accepte les Conditions générales de vente de Banyan",
                },
                uploadFile: {
                    title: "Télécharger des fichiers",
                    subtitle: "Choisissez des fichiers à télécharger depuis votre appareil, ou utilisez le glisser-déposer",
                    selectDrive: "Sélectionner le lecteur",
                    createNewDrive: "Créer un nouveau lecteur",
                    selectFolder: "Sélectionner un dossier",
                    clickToUpload: "Cliquez pour téléverser",
                    orDragAndDrop: "ou glisser-déposer",
                    cancel: "Annuler",
                    upload: "Téléverser",
                    uploadError: "Un problème est survenu avec le téléchargement. Veuillez réessayer",
                    tryAgain: "Réessayer",
                }
            }
        },
    },
    pages: {
        home: {
            allDrives: "Tous les lecteurs",
            upload: "Téléverser",
            newDrive: "Nouveau lecteur",
        },
        registerDevice: {
            title: "Approuver l'accès",
            wantToApproveAccess: "Êtes-vous sûr de vouloir approuver l'accès ",
            cancel: "Annuler",
            approveAccess: "Approuver l'accès",
        },
        trash: {
            trash: "Corbeille",
            trashIsFull: "La corbeille est pleine",
            emptyTrash: "Cliquez pour vider la corbeille",
            trashIsEmpty: "La corbeille est vide",
            clickToEmptyTrash: "Cliquez pour vider la corbeille",
        }
    },
    contexts: {
        fileUpload: {
            softStorageLimit: "Vous approchez de la limite de votre plan de stockage, mettez à niveau pour plus d'espace",
            hardStorageLimit: "Votre demande de stockage dépasse la capacité de stockage, veuillez",
            seePricingPage: "Voir notre page de tarification",
            contactSales: "Contacter les ventes"
        }
    }
}